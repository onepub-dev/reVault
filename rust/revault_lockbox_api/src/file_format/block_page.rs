//! Staged native block page. Metadata is protected separately from data blocks
//! so a partial read need not decrypt/checksum the complete physical page.
use super::{encode_block_frame, BlockFrameDescriptor, BlockFrameReader};
use crate::checked::{read_u16_le, read_u32_le, read_u64_le};
use crate::compression_frame_manifest::{
    decode_compression_frame_manifest, encode_compression_frame_manifest, CompressionFrameManifest,
    CompressionFrameSlice,
};
use crate::creation_options::FormatMode;
use crate::crypto::{open_with_nonce, seal_with_random_nonce, strong_checksum};
use crate::page::{
    page_size_for_stored_len, physical_page_size_from_page_slice, DEFAULT_DATA_PAGE_BYTES,
    PAGE_HEADER_LEN, PAGE_MAGIC,
};
use crate::page_buffer::ZeroizingBytes;
use crate::storage::{Storage, StorageBackend};
use crate::{Error, LockboxId, Result};
use sha2::{Digest, Sha256};

const VERSION: u16 = 3;
const CLEAR: u16 = 1;
const UNPADDED: u16 = 2;
const MAX_METADATA: usize = 1024 * 1024 + BlockFrameDescriptor::ENCODED_LEN;

#[derive(Clone, Copy)]
pub(crate) struct PageIdentity {
    pub(crate) archive: LockboxId,
    pub(crate) page_id: u64,
    pub(crate) sequence: u64,
    pub(crate) mode: FormatMode,
}

/// Immutable encoder output. Cache snapshots share ownership; the last owner
/// wipes its allocation. Only the encoder can construct this type.
pub(crate) struct EncodedBlockPage {
    descriptor: BlockFrameDescriptor,
    bytes: ZeroizingBytes,
}

impl std::fmt::Debug for EncodedBlockPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodedBlockPage")
            .field("len", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

impl EncodedBlockPage {
    pub(crate) fn prepare(
        identity: PageIdentity,
        frame_id: u64,
        compression: u8,
        logical_len: u64,
        stored: &[u8],
        slices: Vec<CompressionFrameSlice>,
        key: &[u8],
    ) -> Result<Self> {
        let (descriptor, bytes) = encode_prepared(
            identity,
            frame_id,
            compression,
            logical_len,
            stored,
            slices,
            key,
        )?;
        Ok(Self {
            descriptor,
            bytes: ZeroizingBytes::new(bytes),
        })
    }
    pub(crate) fn descriptor(&self) -> &BlockFrameDescriptor {
        &self.descriptor
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn metadata_aad(identity: PageIdentity, header: &[u8]) -> Vec<u8> {
    let mut aad = b"revault-native-block-page-v1\0".to_vec();
    aad.extend_from_slice(identity.archive.as_bytes());
    aad.extend_from_slice(&identity.mode.0.to_le_bytes());
    aad.extend_from_slice(&header[..32]);
    aad.extend_from_slice(&header[44..64]);
    aad
}

fn metadata_digest(aad: &[u8], body: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(aad);
    hash.update(body);
    hash.finalize().into()
}

pub(crate) fn encode(
    identity: PageIdentity,
    frame_id: u64,
    input: &[u8],
    slices: Vec<CompressionFrameSlice>,
    key: &[u8],
) -> Result<(BlockFrameDescriptor, Vec<u8>)> {
    if identity.page_id == 0 {
        return Err(Error::CorruptRecord);
    }
    let (descriptor, packet) =
        encode_block_frame(identity.archive, frame_id, identity.mode, input, key)?;
    encode_packet(identity, descriptor, packet, slices, key)
}

/// Wrap a prepared single-pass import result without invoking its data codec.
pub(crate) fn encode_prepared(
    identity: PageIdentity,
    frame_id: u64,
    compression: u8,
    logical_len: u64,
    stored: &[u8],
    slices: Vec<CompressionFrameSlice>,
    key: &[u8],
) -> Result<(BlockFrameDescriptor, Vec<u8>)> {
    if identity.page_id == 0 {
        return Err(Error::CorruptRecord);
    }
    let (descriptor, packet) = super::encode_stored_block_frame(
        identity.archive,
        frame_id,
        identity.mode,
        compression,
        logical_len,
        stored,
        key,
    )?;
    encode_packet(identity, descriptor, packet, slices, key)
}

fn encode_packet(
    identity: PageIdentity,
    descriptor: BlockFrameDescriptor,
    packet: Vec<u8>,
    slices: Vec<CompressionFrameSlice>,
    key: &[u8],
) -> Result<(BlockFrameDescriptor, Vec<u8>)> {
    let packet = ZeroizingBytes::new(packet);
    let manifest = CompressionFrameManifest {
        compression_frame_id: descriptor.frame_id,
        compression: descriptor.compression,
        compression_frame_len: descriptor.logical_len,
        compressed_len: descriptor.stored_len,
        // In page version 3 this field is the stored-index commitment, not a
        // version-2 whole-frame checksum. Dispatch must preserve that distinction.
        compression_frame_digest: descriptor.index_commitment,
        slices,
    };
    let manifest_bytes = ZeroizingBytes::new(encode_compression_frame_manifest(&manifest)?);
    // Retain the existing manifest decoder's permissions/path/extent validation.
    decode_compression_frame_manifest(&manifest_bytes)?;
    let mut metadata = ZeroizingBytes::new(descriptor.encode()?.to_vec());
    metadata.extend_from_slice(&manifest_bytes);
    let (codec, stored) =
        crate::compression::encode_with_compression(&metadata, identity.mode.options().compression);
    let stored = ZeroizingBytes::new(stored);
    let mut body = ZeroizingBytes::new(vec![codec]);
    body.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    body.extend_from_slice(&stored);
    let metadata_len = body.len() + if identity.mode.plaintext() { 32 } else { 16 };
    let body_len = metadata_len
        .checked_add(packet.len())
        .ok_or(Error::CorruptRecord)?;
    let used_len = PAGE_HEADER_LEN
        .checked_add(body_len)
        .ok_or(Error::CorruptRecord)?;
    if used_len > DEFAULT_DATA_PAGE_BYTES {
        return Err(Error::CorruptRecord);
    }
    let physical_len = if identity.mode.unpadded() {
        used_len
    } else {
        page_size_for_stored_len(used_len, DEFAULT_DATA_PAGE_BYTES)?
    };
    let flags = if identity.mode.plaintext() { CLEAR } else { 0 }
        | if identity.mode.unpadded() {
            UNPADDED
        } else {
            0
        };
    let mut header = [0u8; PAGE_HEADER_LEN];
    header[..8].copy_from_slice(PAGE_MAGIC);
    header[8..10].copy_from_slice(&VERSION.to_le_bytes());
    header[10..12].copy_from_slice(&flags.to_le_bytes());
    header[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
    header[16..24].copy_from_slice(&identity.page_id.to_le_bytes());
    header[24..32].copy_from_slice(&identity.sequence.to_le_bytes());
    header[44..48].copy_from_slice(&(body_len as u32).to_le_bytes());
    header[48..56].copy_from_slice(&(physical_len as u64).to_le_bytes());
    header[56..64].copy_from_slice(&(metadata_len as u64).to_le_bytes());
    let aad = metadata_aad(identity, &header);
    let (nonce, protected) = if identity.mode.plaintext() {
        let mut protected = metadata_digest(&aad, &body).to_vec();
        protected.extend_from_slice(&body);
        ([0; 12], protected)
    } else {
        seal_with_random_nonce(&body, key, &aad)?
    };
    let protected = ZeroizingBytes::new(protected);
    header[32..44].copy_from_slice(&nonce);
    let digest = strong_checksum(&header[..64]);
    header[64..].copy_from_slice(&digest);
    let mut page = ZeroizingBytes::new(vec![0; physical_len]);
    page[..PAGE_HEADER_LEN].copy_from_slice(&header);
    page[PAGE_HEADER_LEN..PAGE_HEADER_LEN + metadata_len].copy_from_slice(&protected);
    page[PAGE_HEADER_LEN + metadata_len..used_len].copy_from_slice(&packet);
    Ok((descriptor, std::mem::take(&mut *page)))
}

struct Metadata {
    descriptor: BlockFrameDescriptor,
    manifest: CompressionFrameManifest,
    packet_offset: u64,
    packet_len: usize,
    physical_len: usize,
}

/// A self-consistent scanned page, not proof that its contents were committed
/// or authorized by the archive owner. Recovery must retain commit validation.
#[derive(Debug)]
pub(crate) struct ScannedBlockPage {
    pub(crate) offset: u64,
    pub(crate) page_id: u64,
    pub(crate) sequence: u64,
    pub(crate) physical_len: usize,
    pub(crate) descriptor: BlockFrameDescriptor,
    pub(crate) manifest: CompressionFrameManifest,
}

pub(crate) fn is_native_header(bytes: &[u8]) -> bool {
    bytes.get(..8) == Some(PAGE_MAGIC.as_slice())
        && bytes.get(8..10) == Some(VERSION.to_le_bytes().as_slice())
}

pub(crate) fn scan(
    bytes: &[u8],
    offset: usize,
    archive: LockboxId,
    mode: FormatMode,
    key: &[u8],
) -> Result<ScannedBlockPage> {
    let header = bytes
        .get(
            offset
                ..offset
                    .checked_add(PAGE_HEADER_LEN)
                    .ok_or(Error::CorruptRecord)?,
        )
        .ok_or(Error::Truncated)?;
    let identity = PageIdentity {
        archive,
        mode,
        page_id: read_u64_le(&header[16..24])?,
        sequence: read_u64_le(&header[24..32])?,
    };
    let physical_len = validate_header(header, identity)?;
    let page = bytes
        .get(
            offset
                ..offset
                    .checked_add(physical_len)
                    .ok_or(Error::CorruptRecord)?,
        )
        .ok_or(Error::Truncated)?;
    // Copy only this bounded page, never the entire archive. All returned
    // decoded bytes are wiped immediately after eager full-frame validation.
    let storage = StorageBackend::memory(page.to_vec());
    let metadata = read_metadata(&storage, 0, identity, key)?;
    let frame = BlockFrameReader::open(
        &metadata.descriptor,
        &storage,
        metadata.packet_offset,
        metadata.packet_len,
        key,
    )?;
    let _verified = ZeroizingBytes::new(frame.read(0..metadata.descriptor.logical_len)?);
    Ok(ScannedBlockPage {
        offset: offset as u64,
        page_id: identity.page_id,
        sequence: identity.sequence,
        physical_len,
        descriptor: metadata.descriptor,
        manifest: metadata.manifest,
    })
}

/// Recovery may inspect this metadata before a TOC exists, but must validate
/// all content/commit requirements before reporting a recoverable frame.
fn read_metadata(
    storage: &StorageBackend,
    offset: u64,
    identity: PageIdentity,
    key: &[u8],
) -> Result<Metadata> {
    storage.ensure_current()?;
    let mut header = [0; PAGE_HEADER_LEN];
    storage.read_at_into(offset, &mut header)?;
    let physical_len = validate_header(&header, identity)?;
    let storage_len = storage.len()?;
    if offset
        .checked_add(physical_len as u64)
        .is_none_or(|end| end > storage_len)
    {
        return Err(Error::Truncated);
    }
    let body_len = read_u32_le(&header[44..48])? as usize;
    let metadata_len =
        usize::try_from(read_u64_le(&header[56..64])?).map_err(|_| Error::CorruptRecord)?;
    if !(9 + 16..=MAX_METADATA + 9 + 32).contains(&metadata_len) || metadata_len > body_len {
        return Err(Error::CorruptRecord);
    }
    let mut protected = ZeroizingBytes::new(vec![0; metadata_len]);
    storage.read_at_into(
        offset
            .checked_add(PAGE_HEADER_LEN as u64)
            .ok_or(Error::CorruptRecord)?,
        &mut protected,
    )?;
    let aad = metadata_aad(identity, &header);
    let body = ZeroizingBytes::new(if identity.mode.plaintext() {
        if header[32..44].iter().any(|byte| *byte != 0)
            || protected.len() < 32
            || protected[..32] != metadata_digest(&aad, &protected[32..])
        {
            return Err(Error::CorruptRecord);
        }
        protected[32..].to_vec()
    } else {
        open_with_nonce(&protected, key, &header[32..44], &aad)?
    });
    if body.len() < 9 {
        return Err(Error::CorruptRecord);
    }
    let logical_len = read_u64_le(&body[1..9])?;
    if logical_len > MAX_METADATA as u64
        || logical_len < BlockFrameDescriptor::ENCODED_LEN as u64
        || (identity.mode.options().compression == crate::Compression::None
            && body[0] != crate::compression::COMPRESSION_NONE)
    {
        return Err(Error::CorruptRecord);
    }
    let metadata = ZeroizingBytes::new(crate::compression::decode_compression_frame(
        body[0],
        &body[9..],
        logical_len,
    )?);
    let descriptor = BlockFrameDescriptor::decode(
        &metadata[..BlockFrameDescriptor::ENCODED_LEN],
        identity.archive,
        identity.mode,
    )?;
    let manifest =
        decode_compression_frame_manifest(&metadata[BlockFrameDescriptor::ENCODED_LEN..])?;
    if manifest.compression_frame_id != descriptor.frame_id
        || manifest.compression != descriptor.compression
        || manifest.compression_frame_len != descriptor.logical_len
        || manifest.compressed_len != descriptor.stored_len
        || manifest.compression_frame_digest != descriptor.index_commitment
        || body_len - metadata_len != descriptor.physical_len()?
    {
        return Err(Error::CorruptRecord);
    }
    storage.ensure_current()?;
    Ok(Metadata {
        descriptor,
        manifest,
        packet_offset: offset
            .checked_add((PAGE_HEADER_LEN + metadata_len) as u64)
            .ok_or(Error::CorruptRecord)?,
        packet_len: body_len - metadata_len,
        physical_len,
    })
}

fn validate_header(header: &[u8], identity: PageIdentity) -> Result<usize> {
    let physical_len = physical_page_size_from_page_slice(header)?;
    FormatMode::parse(identity.mode.0)?;
    let expected_flags = if identity.mode.plaintext() { CLEAR } else { 0 }
        | if identity.mode.unpadded() {
            UNPADDED
        } else {
            0
        };
    if identity.mode.0 == 0
        || header.len() != PAGE_HEADER_LEN
        || read_u16_le(&header[8..10])? != VERSION
        || read_u16_le(&header[10..12])? != expected_flags
        || read_u32_le(&header[12..16])? as usize != PAGE_HEADER_LEN
        || identity.page_id == 0
        || read_u64_le(&header[16..24])? != identity.page_id
        || read_u64_le(&header[24..32])? != identity.sequence
        || header[64..] != strong_checksum(&header[..64])
    {
        return Err(Error::CorruptRecord);
    }
    Ok(physical_len)
}

pub(crate) struct Reader<'a> {
    frame: BlockFrameReader<'a>,
    manifest: CompressionFrameManifest,
    physical_len: usize,
    page_offset: u64,
}

pub(crate) fn validate_chunk_reference(
    archive: LockboxId,
    mode: FormatMode,
    total_len: u64,
    chunk: &crate::file_chunk::FileChunk,
) -> Result<PageIdentity> {
    let reference = chunk.block_frame.as_ref().ok_or(Error::CorruptRecord)?;
    let descriptor = &reference.descriptor;
    let [segment] = chunk.segments.as_slice() else {
        return Err(Error::CorruptRecord);
    };
    if descriptor.archive != archive
        || descriptor.mode != mode
        || descriptor.frame_id != chunk.compression_frame_id
        || descriptor.compression != chunk.compression
        || descriptor.logical_len != chunk.compression_frame_len
        || descriptor.stored_len != chunk.compressed_len
        || descriptor.index_commitment != chunk.compression_frame_digest
        || segment.segment_offset != 0
        || segment.segment_len != chunk.compressed_len
        || chunk
            .file_offset
            .checked_add(chunk.len)
            .is_none_or(|end| end > total_len)
        || chunk
            .compression_frame_offset
            .checked_add(chunk.len)
            .is_none_or(|end| end > descriptor.logical_len)
    {
        return Err(Error::CorruptRecord);
    }
    Ok(PageIdentity {
        archive,
        mode,
        page_id: segment.object_id,
        sequence: reference.sequence,
    })
}

/// Recovery validates the entire frame, even when only one packed slice is
/// requested. Its TOC/commit authorization remains the caller's responsibility.
pub(crate) fn read_recovery_chunk(
    bytes: &[u8],
    archive: LockboxId,
    mode: FormatMode,
    total_len: u64,
    chunk: &crate::file_chunk::FileChunk,
    key: &[u8],
) -> Result<Vec<u8>> {
    let identity = validate_chunk_reference(archive, mode, total_len, chunk)?;
    let segment = &chunk.segments[0];
    let offset = usize::try_from(segment.page_offset).map_err(|_| Error::CorruptRecord)?;
    let header = bytes
        .get(
            offset
                ..offset
                    .checked_add(PAGE_HEADER_LEN)
                    .ok_or(Error::CorruptRecord)?,
        )
        .ok_or(Error::Truncated)?;
    let physical_len = validate_header(header, identity)?;
    if physical_len as u64 != segment.page_len {
        return Err(Error::CorruptRecord);
    }
    let page = bytes
        .get(
            offset
                ..offset
                    .checked_add(physical_len)
                    .ok_or(Error::CorruptRecord)?,
        )
        .ok_or(Error::Truncated)?;
    let storage = StorageBackend::memory(page.to_vec());
    let descriptor = &chunk
        .block_frame
        .as_ref()
        .ok_or(Error::CorruptRecord)?
        .descriptor;
    let reader = Reader::open(&storage, 0, identity, descriptor, physical_len, key)?;
    reader.validate_slice(chunk, total_len)?;
    let mut decoded = ZeroizingBytes::new(reader.read(0..descriptor.logical_len)?);
    let start =
        usize::try_from(chunk.compression_frame_offset).map_err(|_| Error::CorruptRecord)?;
    let len = usize::try_from(chunk.len).map_err(|_| Error::CorruptRecord)?;
    let end = start.checked_add(len).ok_or(Error::CorruptRecord)?;
    if end > decoded.len() {
        return Err(Error::CorruptRecord);
    }
    decoded.copy_within(start..end, 0);
    zeroize::Zeroize::zeroize(&mut decoded[len..]);
    decoded.truncate(len);
    Ok(std::mem::take(&mut *decoded))
}

impl<'a> Reader<'a> {
    pub(crate) fn validate_slice(
        &self,
        chunk: &crate::file_chunk::FileChunk,
        expected_total_len: u64,
    ) -> Result<()> {
        if self
            .manifest
            .slice_for(
                &chunk.stored_path,
                chunk.file_offset,
                chunk.compression_frame_offset,
                chunk.len,
            )
            .is_none_or(|slice| slice.total_len != 0 && slice.total_len != expected_total_len)
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    pub(crate) fn open(
        storage: &'a StorageBackend,
        offset: u64,
        identity: PageIdentity,
        expected: &BlockFrameDescriptor,
        expected_page_len: usize,
        key: &[u8],
    ) -> Result<Self> {
        let metadata = read_metadata(storage, offset, identity, key)?;
        if metadata.descriptor != *expected || metadata.physical_len != expected_page_len {
            return Err(Error::CorruptRecord);
        }
        let frame = BlockFrameReader::open(
            &metadata.descriptor,
            storage,
            metadata.packet_offset,
            metadata.packet_len,
            key,
        )?;
        Ok(Self {
            frame,
            manifest: metadata.manifest,
            physical_len: metadata.physical_len,
            page_offset: offset,
        })
    }

    pub(crate) fn read(&self, range: std::ops::Range<u64>) -> Result<Vec<u8>> {
        self.ensure_current()?;
        self.frame.read(range)
    }

    pub(crate) fn ensure_current(&self) -> Result<()> {
        self.frame.storage.ensure_current()?;
        let len = self.frame.storage.len()?;
        if self
            .page_offset
            .checked_add(self.physical_len as u64)
            .is_none_or(|end| end > len)
        {
            return Err(Error::Truncated);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Compression, EncryptionMode, LockboxFormatOptions, LockboxPath, SigningMode, SizePadding,
    };

    #[test]
    fn native_scanning_skips_embedded_legacy_records_and_keeps_following_pages() {
        use crate::page::{PageObject, PageObjectKind};
        use crate::{Encryption, Lockbox, LockboxCreateOptions, Signing};
        for size_padding in [SizePadding::Default, SizePadding::None] {
            let archive = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                compression: Compression::None,
                size_padding,
                ..LockboxCreateOptions::new(Encryption::None, Signing::None)
            })
            .unwrap();
            let mut bytes = archive.to_bytes();
            let header = crate::file_format::read_header(&bytes).unwrap();
            let object = PageObject::new(PageObjectKind::FileData, 777, vec![1, 2, 3]);
            let objects = vec![object];
            let page_len = crate::page::page_size_for_encoded_objects_with_format(
                &objects,
                header.format_mode,
            )
            .unwrap();
            let embedded = crate::page::encode_page_with_format(
                page_len,
                header.lockbox_id,
                778,
                779,
                &[0; 32],
                &objects,
                header.format_mode,
            )
            .unwrap();
            let identity = PageIdentity {
                archive: header.lockbox_id,
                mode: header.format_mode,
                page_id: 900,
                sequence: 901,
            };
            let (_, native) = encode(
                identity,
                899,
                &embedded,
                vec![CompressionFrameSlice {
                    path: LockboxPath::new("/contains-page-magic").unwrap(),
                    permissions: 0o640,
                    total_len: embedded.len() as u64,
                    file_offset: 0,
                    compression_frame_offset: 0,
                    len: embedded.len() as u64,
                }],
                &[0; 32],
            )
            .unwrap();
            let offset = bytes.len();
            bytes.extend_from_slice(&native);
            let following = bytes.len();
            bytes.extend_from_slice(&embedded);
            let scan = crate::page::scan_page_records(&bytes, header.lockbox_id, &[0; 32]);
            assert_eq!(scan.corrupt_records, 0);
            assert_eq!(scan.native_pages.len(), 1);
            assert_eq!(scan.native_pages[0].offset, offset as u64);
            let records = scan
                .records
                .iter()
                .filter(|record| record.object_id == 777)
                .collect::<Vec<_>>();
            assert_eq!(
                records.len(),
                1,
                "native content must not produce phantom legacy records"
            );
            assert_eq!(records[0].offset, following as u64);
        }
    }

    fn identity(encrypted: bool, signed: bool, compressed: bool, unpadded: bool) -> PageIdentity {
        PageIdentity {
            archive: LockboxId::from_bytes([71; 16]),
            page_id: 41,
            sequence: 43,
            mode: FormatMode::new(LockboxFormatOptions {
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
                size_padding: if unpadded {
                    SizePadding::None
                } else {
                    SizePadding::Default
                },
            }),
        }
    }

    fn slices(len: usize) -> Vec<CompressionFrameSlice> {
        let first = len / 3;
        vec![
            CompressionFrameSlice {
                path: LockboxPath::new("/first.bin").unwrap(),
                permissions: 0o640,
                total_len: first as u64,
                file_offset: 0,
                compression_frame_offset: 0,
                len: first as u64,
            },
            CompressionFrameSlice {
                path: LockboxPath::new("/nested/second.bin").unwrap(),
                permissions: 0o600,
                total_len: (len - first + 17) as u64,
                file_offset: 17,
                compression_frame_offset: first as u64,
                len: (len - first) as u64,
            },
        ]
    }

    #[test]
    fn block_page_roundtrips_recovery_manifest_and_padding_in_all_modes() {
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compressed in [false, true] {
                    for unpadded in [false, true] {
                        let identity = identity(encrypted, signed, compressed, unpadded);
                        for len in [0, 128, 65539] {
                            let input: Vec<_> = (0..len).map(|n| (n % 251) as u8).collect();
                            let slices = slices(len);
                            let (descriptor, page) =
                                encode(identity, 31, &input, slices.clone(), &[53; 32]).unwrap();
                            let used =
                                PAGE_HEADER_LEN + read_u32_le(&page[44..48]).unwrap() as usize;
                            if unpadded {
                                assert_eq!(page.len(), used);
                            } else {
                                assert_eq!(page.len() % 1024, 0);
                                assert!(page.len() >= used);
                            }
                            assert!(page[used..].iter().all(|byte| *byte == 0));
                            assert_eq!(
                                physical_page_size_from_page_slice(&page).unwrap(),
                                page.len()
                            );
                            // The existing reader must not silently treat new
                            // block pages as ordinary whole-page encrypted data.
                            assert!(crate::page::decode_page_with_format(
                                &page,
                                identity.archive,
                                &[53; 32],
                                identity.mode
                            )
                            .is_err());
                            let mut bytes = vec![79; 317];
                            bytes.extend_from_slice(&page);
                            bytes.extend_from_slice(&[83; 211]);
                            let storage = StorageBackend::memory(bytes);
                            let metadata =
                                read_metadata(&storage, 317, identity, &[53; 32]).unwrap();
                            assert_eq!(metadata.manifest.slices, slices);
                            assert_eq!(metadata.descriptor, descriptor);
                            let reader = Reader::open(
                                &storage,
                                317,
                                identity,
                                &descriptor,
                                page.len(),
                                &[53; 32],
                            )
                            .unwrap();
                            assert_eq!(reader.manifest.slices, slices);
                            assert_eq!(reader.read(0..len as u64).unwrap(), input);
                            for slice in &slices {
                                let start = slice.compression_frame_offset;
                                assert_eq!(
                                    reader.read(start..start + slice.len).unwrap(),
                                    input[start as usize..(start + slice.len) as usize]
                                );
                            }
                            assert_eq!(reader.read(0..0).unwrap(), b"");
                            assert!(Reader::open(
                                &storage,
                                317,
                                identity,
                                &descriptor,
                                page.len() + 1,
                                &[53; 32]
                            )
                            .is_err());
                            let mut wrong = descriptor.clone();
                            wrong.frame_id += 1;
                            assert!(Reader::open(
                                &storage,
                                317,
                                identity,
                                &wrong,
                                page.len(),
                                &[53; 32]
                            )
                            .is_err());
                            assert_eq!(storage.read_at(0, 317).unwrap(), vec![79; 317]);
                            assert_eq!(
                                storage.read_at(317 + page.len() as u64, 211).unwrap(),
                                vec![83; 211]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn block_page_rejects_header_metadata_and_extent_corruption_before_use() {
        for encrypted in [false, true] {
            for unpadded in [false, true] {
                let identity = identity(encrypted, false, false, unpadded);
                let (descriptor, page) =
                    encode(identity, 31, &[37; 32769], slices(32769), &[53; 32]).unwrap();
                for byte in [8, 10, 12, 16, 24, 32, 44, 48, 56, 64, PAGE_HEADER_LEN] {
                    let mut corrupt = page.clone();
                    corrupt[byte] ^= 1;
                    // Do not let a mere public checksum obscure missing AEAD or
                    // expected-coordinate checks: repair it after header edits.
                    if byte < 64 {
                        let digest = strong_checksum(&corrupt[..64]);
                        corrupt[64..96].copy_from_slice(&digest);
                    }
                    assert!(
                        Reader::open(
                            &StorageBackend::memory(corrupt),
                            0,
                            identity,
                            &descriptor,
                            page.len(),
                            &[53; 32]
                        )
                        .is_err(),
                        "byte {byte}"
                    );
                }
                let mut huge = page.clone();
                huge[56..64].copy_from_slice(&u64::MAX.to_le_bytes());
                let digest = strong_checksum(&huge[..64]);
                huge[64..96].copy_from_slice(&digest);
                assert!(
                    read_metadata(&StorageBackend::memory(huge), 0, identity, &[53; 32]).is_err()
                );
                let truncated = StorageBackend::memory(page[..page.len() - 1].to_vec());
                assert!(
                    Reader::open(&truncated, 0, identity, &descriptor, page.len(), &[53; 32])
                        .is_err()
                );
                if encrypted {
                    assert!(Reader::open(
                        &StorageBackend::memory(page.clone()),
                        0,
                        identity,
                        &descriptor,
                        page.len(),
                        &[54; 32]
                    )
                    .is_err());
                    assert!(!page
                        .windows(b"/nested/second.bin".len())
                        .any(|w| w == b"/nested/second.bin"));
                }
                let mut swapped = identity;
                swapped.mode.0 ^= 2;
                assert!(
                    read_metadata(&StorageBackend::memory(page), 0, swapped, &[53; 32]).is_err()
                );
            }
        }
    }

    #[test]
    fn block_page_cached_reader_rejects_lost_padding_and_corrupt_recovery_data() {
        let root = std::env::temp_dir().join(format!(
            "revault-block-page-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        for encrypted in [false, true] {
            let identity = identity(encrypted, false, false, false);
            let (descriptor, page) =
                encode(identity, 31, &[37; 128], slices(128), &[53; 32]).unwrap();
            let used = PAGE_HEADER_LEN + read_u32_le(&page[44..48]).unwrap() as usize;
            assert!(page.len() > used);
            let path = root.join(format!("{encrypted}"));
            let storage = StorageBackend::create_file(&path, &page).unwrap();
            let reader =
                Reader::open(&storage, 0, identity, &descriptor, page.len(), &[53; 32]).unwrap();
            assert_eq!(reader.read(0..128).unwrap(), [37; 128]);
            let mut damaged = storage.clone();
            damaged.truncate(page.len() as u64 - 1).unwrap();
            assert!(reader.read(0..1).is_err());
            assert!(reader.read(0..0).is_err());

            let mut damaged = page;
            let metadata = read_metadata(
                &StorageBackend::memory(damaged.clone()),
                0,
                identity,
                &[53; 32],
            )
            .unwrap();
            damaged[metadata.packet_offset as usize + descriptor.index_len().unwrap()] ^= 1;
            let storage = StorageBackend::memory(damaged);
            // Metadata can be intact while data is not. Recovery must use the
            // frame verifier, not treat manifest decoding as successful recovery.
            let metadata = read_metadata(&storage, 0, identity, &[53; 32]).unwrap();
            let frame = BlockFrameReader::open(
                &metadata.descriptor,
                &storage,
                metadata.packet_offset,
                metadata.packet_len,
                &[53; 32],
            )
            .unwrap();
            assert!(frame.read(0..128).is_err());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn block_page_rejects_invalid_manifest_and_rechecksummed_metadata_bombs() {
        let identity = identity(false, false, false, true);
        let mut invalid = slices(128);
        invalid[0].len = 129;
        assert!(encode(identity, 31, &[37; 128], invalid, &[53; 32]).is_err());
        let mut invalid = slices(128);
        invalid[0].permissions = u32::MAX;
        assert!(encode(identity, 31, &[37; 128], invalid, &[53; 32]).is_err());
        let (descriptor, mut page) =
            encode(identity, 31, &[37; 128], slices(128), &[53; 32]).unwrap();
        // Forge a huge metadata expansion length and recompute the public digest.
        // The bound must reject it before allocating, even without AEAD.
        let metadata_len = read_u64_le(&page[56..64]).unwrap() as usize;
        let body_start = PAGE_HEADER_LEN + 32;
        page[body_start + 1..body_start + 9].copy_from_slice(&u64::MAX.to_le_bytes());
        let aad = metadata_aad(identity, &page[..PAGE_HEADER_LEN]);
        let digest = metadata_digest(&aad, &page[body_start..PAGE_HEADER_LEN + metadata_len]);
        page[PAGE_HEADER_LEN..body_start].copy_from_slice(&digest);
        let page_len = page.len();
        assert!(Reader::open(
            &StorageBackend::memory(page),
            0,
            identity,
            &descriptor,
            page_len,
            &[53; 32]
        )
        .is_err());
    }
}
