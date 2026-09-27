//! Read-only feasibility model, NOT an archive encoding or publication protocol.
//! Repack independently authenticated fragments without coupling aggregate decoded
//! bytes to the per-fragment restart bound. Measure a compact private catalogue
//! with shared physical-pack records. No source or destination archive is written.
use super::*;
use crate::compression::{encode_with_compression, COMPRESSION_NONE};
use crate::crypto::strong_checksum;
use crate::file_format::allocation_map::Extent;
use crate::page_buffer::ZeroizingBytes;
use serde_json::{json, Value};
use zstd_complete::decoding::StaticDecoderWorkspace;

const REGION: usize = 65536;
const PRIVATE_SLOT: usize = 48 * 1024;
const MAX_MEMBERS: usize = 4096;
struct Fragment {
    source: Extent,
    descriptor: Descriptor,
    pack: usize,
    relative: usize,
}
struct File {
    path: Zeroizing<Vec<u8>>,
    info: FileInfo,
    fragments: Vec<Fragment>,
}
struct Pack {
    extent: Extent,
    padding_digest: [u8; 32],
}
fn uint(out: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        out.push(n as u8 | 128);
        n >>= 7;
    }
    out.push(n as u8);
}
fn finish_pack(
    codec: &Codec,
    bytes: &mut ZeroizingBytes,
    packs: &mut Vec<Pack>,
    end: &mut u64,
) -> Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }
    let padded = codec.pack_padded_len(bytes.len())?;
    let padding = codec.pack_padding(padded - bytes.len())?;
    let padding_digest = strong_checksum(&padding);
    bytes.extend_from_slice(&padding);
    let extent = Extent {
        start: *end,
        len: bytes.len() as u64,
        digest: strong_checksum(bytes),
    };
    *end += extent.len;
    packs.push(Pack {
        extent,
        padding_digest,
    });
    crate::page_buffer::zeroize_bytes(bytes);
    bytes.clear();
    Ok(())
}

pub(super) fn project<S: Storage>(archive: &mut Files<S>) -> Result<Value> {
    archive.audit_with(true)?;
    let mut files = Vec::<File>::new();
    let mut identities = BTreeMap::new();
    let mut chunks = 0;
    archive.index.visit(
        &archive.storage,
        archive.anchor.index,
        archive.anchor.sealed_len,
        |entry| {
            let owned = OwnedRecord::decode(&entry.value)?;
            match entry.namespace {
                FILE => {
                    if files.len() >= 1024 {
                        return Err(Error::SecurityLimitExceeded(
                            "cost model: at most 1024 files".into(),
                        ));
                    }
                    let info = FileInfo::decode(&owned.metadata, &archive.codec)?;
                    identities.insert(info.id, files.len());
                    files.push(File {
                        path: entry.key,
                        info,
                        fragments: Vec::new(),
                    });
                }
                CHUNK => {
                    chunks += 1;
                    if chunks > 4096 {
                        return Err(Error::SecurityLimitExceeded(
                            "cost model: at most 4096 fragments".into(),
                        ));
                    }
                    let slice = Slice::decode(&owned.metadata)?;
                    let file = identities
                        .get(&slice.descriptor.object)
                        .ok_or(Error::CorruptRecord)?;
                    files[*file].fragments.push(Fragment {
                        source: slice.physical(owned.extents[0])?,
                        descriptor: slice.descriptor,
                        pack: 0,
                        relative: 0,
                    });
                }
                _ => return Err(Error::CorruptRecord),
            }
            Ok(())
        },
    )?;
    let unit = archive.codec.logical_unit(MAX_LOGICAL)?;
    let mut packed = ZeroizingBytes::new(Vec::with_capacity(unit + REGION));
    let mut packs = Vec::new();
    let mut end = (2 * REGION) as u64;
    let mut members = 0;
    let mut logical = 0u64;
    let mut largest_pack_logical = 0;
    let mut largest_fragment = 0;
    let mut total_stored = 0;
    for file in &mut files {
        for (ordinal, fragment) in file.fragments.iter_mut().enumerate() {
            // Reconstruct every omitted descriptor field from its committed file
            // header and ordinal; this must preserve the complete current AAD.
            let encoded = fragment.descriptor.encode();
            let mut reconstructed = [0u8; 64];
            reconstructed[..8].copy_from_slice(b"RV4DAT01");
            reconstructed[8..24].copy_from_slice(&file.info.id);
            reconstructed[24..32].copy_from_slice(&(ordinal as u64).to_le_bytes());
            let offset = ordinal as u64 * file.info.unit as u64;
            reconstructed[32..40].copy_from_slice(&offset.to_le_bytes());
            let len = (file.info.len - offset).min(file.info.unit as u64) as u32;
            reconstructed[40..44].copy_from_slice(&len.to_le_bytes());
            reconstructed[44..53].copy_from_slice(&encoded[44..53]);
            if Descriptor::decode(&reconstructed)? != fragment.descriptor {
                return Err(Error::CorruptRecord);
            }
            let stored = ZeroizingBytes::new(
                archive
                    .storage
                    .read_at(fragment.source.start, fragment.source.len as usize)?,
            );
            if stored.len() != fragment.source.len as usize
                || strong_checksum(&stored) != fragment.source.digest
            {
                return Err(Error::CorruptRecord);
            }
            if !packed.is_empty() && (packed.len() + stored.len() > unit || members == MAX_MEMBERS)
            {
                finish_pack(&archive.codec, &mut packed, &mut packs, &mut end)?;
                members = 0;
                logical = 0;
            }
            fragment.pack = packs.len();
            fragment.relative = packed.len();
            packed.extend_from_slice(&stored);
            members += 1;
            logical += len as u64;
            largest_pack_logical = largest_pack_logical.max(logical);
            largest_fragment = largest_fragment.max(len);
            total_stored += stored.len() as u64;
        }
    }
    finish_pack(&archive.codec, &mut packed, &mut packs, &mut end)?;

    let mut body = ZeroizingBytes::new(Vec::with_capacity(1024 * 1024));
    body.extend_from_slice(b"RV4COST1");
    uint(&mut body, files.len() as u64);
    for file in &files {
        // Reserve was made before private bytes were copied. Refuse before any
        // operation could grow/reallocate that private buffer beyond its budget.
        if body.len() + file.path.len() + 128 + file.fragments.len() * 64 > 1024 * 1024 {
            return Err(Error::SecurityLimitExceeded(
                "cost model catalogue budget".into(),
            ));
        }
        body.push(1); // File kind; real directory/symlink semantics remain absent.
        body.extend_from_slice(&0o644u32.to_le_bytes()); // Explicit synthetic permission cost.
        uint(&mut body, file.path.len() as u64);
        body.extend_from_slice(&file.path);
        body.extend_from_slice(&file.info.id);
        uint(&mut body, file.info.len);
        uint(&mut body, file.info.unit as u64);
        body.extend_from_slice(&file.info.digest);
        uint(&mut body, file.fragments.len() as u64);
        for fragment in &file.fragments {
            uint(&mut body, fragment.pack as u64);
            uint(&mut body, fragment.relative as u64);
            // Encoded length, stored length and codec remain explicit. Logical
            // binding fields are reconstructed and checked above, not discarded.
            body.extend_from_slice(&fragment.descriptor.encode()[44..53]);
            body.extend_from_slice(&fragment.source.digest);
        }
    }
    if body.len() + 20 + packs.len() * 96 > 1024 * 1024 {
        return Err(Error::SecurityLimitExceeded(
            "cost model pack-table budget".into(),
        ));
    }
    uint(&mut body, packs.len() as u64);
    for pack in &packs {
        body.extend_from_slice(&pack.extent.start.to_le_bytes());
        uint(&mut body, pack.extent.len);
        body.extend_from_slice(&pack.extent.digest);
        body.extend_from_slice(&pack.padding_digest);
    }
    uint(&mut body, 0); // Fresh compacted image: no external free/pending ranges.
    if body.len() > 1024 * 1024 {
        return Err(Error::SecurityLimitExceeded(
            "cost model catalogue budget".into(),
        ));
    }
    let (codec, encoded) = encode_with_compression(&body, crate::Compression::default());
    let encoded = ZeroizingBytes::new(encoded);
    let decoder_window = body.len().next_power_of_two().max(REGION);
    if codec == COMPRESSION_NONE {
        if encoded.as_slice() != body.as_slice() {
            return Err(Error::CorruptRecord);
        }
    } else {
        let required = StaticDecoderWorkspace::required_size(decoder_window, 0)
            .map_err(|_| Error::CorruptRecord)?;
        let mut scratch = ZeroizingBytes::new(vec![0; required]);
        let mut output = ZeroizingBytes::new(vec![0; body.len()]);
        let mut decoder = StaticDecoderWorkspace::new(&mut scratch, decoder_window, 0)
            .map_err(|_| Error::CorruptRecord)?;
        if decoder
            .decode_into(&encoded, &mut output)
            .map_err(|_| Error::CorruptRecord)?
            != body.len()
            || output.as_slice() != body.as_slice()
        {
            return Err(Error::CorruptRecord);
        }
    }
    // 44-byte page header and 16-byte tag budget is conservative for plaintext.
    let fits = body.len() <= REGION && encoded.len() + 60 <= PRIVATE_SLOT;
    let payload = end - (2 * REGION) as u64;
    Ok(json!({
        "kind":"whole_layout_cost_model","source_archive_bytes":archive.storage.len()?,
        "files":files.len(),"fragments":chunks,"physical_packs":packs.len(),
        "stored_fragment_bytes":total_stored,"pack_padding_bytes":payload-total_stored,
        "payload_allocation_bytes":payload,"largest_fragment_decoded_bytes":largest_fragment,
        "largest_pack_logical_bytes":largest_pack_logical,"max_members_per_pack":MAX_MEMBERS,
        "catalogue_plain_bytes":body.len(),"catalogue_encoded_bytes":encoded.len(),
        "catalogue_codec":codec,"catalogue_decoder_window":decoder_window,"inline_private_slot_bytes":PRIVATE_SLOT,
        "control_regions":2,"control_region_bytes":REGION,"inline_catalogue_fits":fits,
        "projected_compacted_bytes":if fits { Some(end) } else { None },
        "metadata_roundtrip_verified":true,"descriptor_binding_preserved":true,
        "scope":"cost-only, no persisted archive, bootstrap/overflow journal/ownership/publication protocol not implemented"
    }))
}
