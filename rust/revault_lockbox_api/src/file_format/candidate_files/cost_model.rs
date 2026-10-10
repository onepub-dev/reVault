//! Feasibility model and bounded streaming hooks for the fresh-image experiment.
//! Repack independently authenticated fragments without coupling aggregate decoded
//! bytes to the per-fragment restart bound. Measure a compact private catalogue
//! with shared physical-pack records. `project` remains read-only; `project_into`
//! emits verified packs and the bounded body to caller-owned sinks. Publication
//! and cleanup belong to the fresh-image builder, not this model.
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

/// Audited encoded fragments and bounded metadata, without a serialized catalogue.
/// Payload copying has completed, but publication and fresh-destination cleanup
/// remain the caller's responsibility.
pub(super) struct Repacked {
    files: Vec<File>,
    packs: Vec<Pack>,
    end: u64,
    chunks: usize,
    total_stored: u64,
    largest_pack_logical: u64,
    largest_fragment: u32,
}
impl Repacked {
    pub(super) fn end(&self) -> u64 {
        self.end
    }

    pub(super) fn into_catalogue(self, codec: &Codec) -> Result<super::dense_catalogue::Catalogue> {
        use super::dense_catalogue;
        let files = self
            .files
            .into_iter()
            .map(|file| dense_catalogue::File {
                path: file.path,
                permissions: 0o644,
                info: file.info,
                fragments: file
                    .fragments
                    .into_iter()
                    .map(|fragment| dense_catalogue::Fragment {
                        descriptor: fragment.descriptor,
                        pack: fragment.pack,
                        relative: fragment.relative,
                        digest: fragment.source.digest,
                    })
                    .collect(),
            })
            .collect();
        let packs = self
            .packs
            .into_iter()
            .map(|pack| dense_catalogue::Pack {
                extent: pack.extent,
                padding_digest: pack.padding_digest,
                used: 0,
            })
            .collect();
        dense_catalogue::Catalogue::from_repacked(files, packs, codec, self.end)
    }
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
    emit: &mut impl FnMut(Extent, &[u8]) -> Result<()>,
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
    emit(extent, bytes)?;
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
    project_into(archive, |_, _| Ok(()), |_| Ok(()))
}
pub(super) fn project_into<S: Storage>(
    archive: &mut Files<S>,
    emit: impl FnMut(Extent, &[u8]) -> Result<()>,
    mut catalogue: impl FnMut(&[u8]) -> Result<()>,
) -> Result<Value> {
    let Repacked {
        files,
        packs,
        end,
        chunks,
        total_stored,
        largest_pack_logical,
        largest_fragment,
    } = repack_into(archive, emit)?;

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
    let (codec, encoded) =
        encode_with_compression(&body, archive.anchor.mode.options().compression);
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
    // The shared envelope needs a 44-byte header, 12-byte private length/codec
    // header and a 16-byte tag. The tag budget is conservative for plaintext.
    let fits = body.len() <= REGION && encoded.len() + 72 <= PRIVATE_SLOT;
    let payload = end - (2 * REGION) as u64;
    if fits {
        catalogue(&body)?;
    }
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

pub(super) fn repack_into<S: Storage>(
    archive: &mut Files<S>,
    emit: impl FnMut(Extent, &[u8]) -> Result<()>,
) -> Result<Repacked> {
    repack_bounded(archive, emit, false)
}

pub(super) fn repack_tree_into<S: Storage>(
    archive: &mut Files<S>,
    metadata: &[super::dense_catalogue::Metadata],
    emit: impl FnMut(Extent, &[u8]) -> Result<()>,
) -> Result<Repacked> {
    use crate::file_format::metadata_budget as memory;
    // Preflight precedes the source audit's retained file/coverage maps and all
    // copying. One pack per fragment is a conservative bound on repack output.
    let mut budget = memory::Budget::new(memory::TYPED_BYTES);
    archive.index.visit(
        &archive.storage,
        archive.anchor.index,
        archive.anchor.sealed_len,
        |entry| {
            match entry.namespace {
                FILE => {
                    budget.take(memory::FILE)?;
                    budget.paths(entry.key.len())?;
                }
                CHUNK => budget.take(memory::FRAGMENT + memory::PACK)?,
                _ => return Err(Error::CorruptRecord),
            }
            Ok(())
        },
    )?;
    for entry in metadata {
        if entry.entry.kind != crate::LockboxEntryKind::File {
            budget.take(memory::FILE)?;
            budget.paths(entry.entry.path.as_str().len())?;
            budget.paths(
                entry
                    .target
                    .as_ref()
                    .map_or(0, |target| target.as_str().len()),
            )?;
        }
    }
    repack_bounded(archive, emit, true)
}

fn repack_bounded<S: Storage>(
    archive: &mut Files<S>,
    mut emit: impl FnMut(Extent, &[u8]) -> Result<()>,
    tree: bool,
) -> Result<Repacked> {
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
                    if !tree && files.len() >= 1024 {
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
                    if !tree && chunks > 4096 {
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
                finish_pack(&archive.codec, &mut packed, &mut packs, &mut end, &mut emit)?;
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
    finish_pack(&archive.codec, &mut packed, &mut packs, &mut end, &mut emit)?;

    Ok(Repacked {
        files,
        packs,
        end,
        chunks,
        total_stored,
        largest_pack_logical,
        largest_fragment,
    })
}
