//! Read-only whole-layout feasibility model. No writer or reader accepts these
//! pages. Measure real encoded records, hybrid publication occupancy and the
//! old/new leaf coexistence budget before implementing a paged catalogue.
use super::dense_catalogue::{Catalogue, File};
use super::*;
use crate::compression::{encode_with_compression, COMPRESSION_NONE};
use crate::crypto::{open_with_nonce, seal_with_random_nonce, strong_checksum};
use crate::page_buffer::ZeroizingBytes;
use serde_json::{json, Value};
use zstd_complete::decoding::StaticDecoderWorkspace;
const REGION: u64 = 65536;
const PRIVATE_START: u64 = 16384;
const PRIVATE_BYTES: usize = 49152;
const ROOT_START: usize = 6144;
const ROOT_BYTES: usize = 2016; // Publication footer retains its independent checksum.
const LEAF_RECORDS: usize = 32;
const MAX_PLAIN: usize = 65536;
const MAX_LEAF_PLAIN: usize = 16384;
const QUANTUM: usize = 256;
fn uint(out: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        out.push(n as u8 | 128);
        n >>= 7;
    }
    out.push(n as u8);
}
struct FrameCodec {
    archive: LockboxId,
    mode: FormatMode,
    key: Option<Zeroizing<[u8; 32]>>,
}
impl FrameCodec {
    fn new(archive: LockboxId, mode: FormatMode, key: Option<&[u8]>) -> Result<Self> {
        if mode.plaintext() != key.is_none() || key.is_some_and(|key| key.len() != 32) {
            return Err(Error::InvalidKey);
        }
        let key = key.map(|key| {
            let mut out = Zeroizing::new([0; 32]);
            hkdf::Hkdf::<Sha256>::new(Some(archive.as_bytes()), key)
                .expand(b"revault-paged-metadata-cost-model-v1\0", &mut *out)
                .expect("fixed key size");
            out
        });
        Ok(Self { archive, mode, key })
    }
    fn encode(&self, plain: &[u8], fixed: Option<usize>) -> Result<(ZeroizingBytes, usize)> {
        if plain.len() > MAX_PLAIN {
            return Err(Error::SecurityLimitExceeded(
                "paged model decoded limit".into(),
            ));
        }
        let (kind, encoded) = encode_with_compression(plain, self.mode.options().compression);
        let encoded = ZeroizingBytes::new(encoded);
        let overhead = 56 + if self.key.is_some() { 16 } else { 0 };
        let required = encoded.len() + overhead;
        let len = fixed.unwrap_or(required.div_ceil(QUANTUM) * QUANTUM);
        if len < required {
            return Err(Error::SecurityLimitExceeded(format!(
                "paged model frame requires {required} bytes"
            )));
        }
        let body_len = len - 44 - if self.key.is_some() { 16 } else { 0 };
        let mut body = ZeroizingBytes::new(Vec::with_capacity(body_len));
        body.extend_from_slice(&(plain.len() as u32).to_le_bytes());
        body.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        body.push(kind);
        body.extend_from_slice(&[0; 3]);
        body.extend_from_slice(&encoded);
        body.resize(body_len, 0);
        let mut out = ZeroizingBytes::new(Vec::with_capacity(len));
        out.extend_from_slice(b"RV4MPC01");
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&self.mode.0.to_le_bytes());
        out.extend_from_slice(self.archive.as_bytes());
        if let Some(key) = &self.key {
            let (nonce, encrypted) = seal_with_random_nonce(&body, key.as_slice(), &out)?;
            out.extend_from_slice(&nonce);
            out.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());
            out.extend_from_slice(&encrypted);
        } else {
            out.extend_from_slice(&[0; 12]);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&body);
        }
        if out.len() != len {
            return Err(Error::CorruptRecord);
        }
        // Round-trip actual AEAD and compression bytes, including zero padding.
        let recovered = if let Some(key) = &self.key {
            ZeroizingBytes::new(open_with_nonce(
                &out[44..],
                key.as_slice(),
                &out[28..40],
                &out[..28],
            )?)
        } else {
            ZeroizingBytes::new(out[44..].to_vec())
        };
        if recovered.as_slice() != body.as_slice() {
            return Err(Error::CorruptRecord);
        }
        let compressed = &recovered[12..12 + encoded.len()];
        if kind == COMPRESSION_NONE {
            if compressed != plain {
                return Err(Error::CorruptRecord);
            }
        } else {
            let required = StaticDecoderWorkspace::required_size(MAX_PLAIN, 0)
                .map_err(|_| Error::CorruptRecord)?;
            let mut scratch = ZeroizingBytes::new(vec![0; required]);
            let mut output = ZeroizingBytes::new(vec![0; plain.len()]);
            let mut decoder = StaticDecoderWorkspace::new(&mut scratch, MAX_PLAIN, 0)
                .map_err(|_| Error::CorruptRecord)?;
            if decoder
                .decode_into(compressed, &mut output)
                .map_err(|_| Error::CorruptRecord)?
                != plain.len()
                || output.as_slice() != plain
            {
                return Err(Error::CorruptRecord);
            }
        }
        Ok((out, required))
    }
}
fn file_record(file: &File) -> Result<ZeroizingBytes> {
    let uint_len = |n: u64| ((64 - n.leading_zeros()).max(1) as usize).div_ceil(7);
    let len = 1
        + 4
        + uint_len(file.path.len() as u64)
        + file.path.len()
        + 16
        + uint_len(file.info.len)
        + uint_len(file.info.unit as u64)
        + 32
        + uint_len(file.fragments.len() as u64)
        + file
            .fragments
            .iter()
            .map(|fragment| {
                uint_len(fragment.pack as u64) + uint_len(fragment.relative as u64) + 41
            })
            .sum::<usize>();
    if len > MAX_PLAIN {
        return Err(Error::SecurityLimitExceeded(
            "paged model file record budget".into(),
        ));
    }
    // Input comes from the validated compact catalogue; preserve every field.
    let mut out = ZeroizingBytes::new(Vec::with_capacity(MAX_PLAIN));
    out.push(1);
    out.extend_from_slice(&file.permissions.to_le_bytes());
    uint(&mut out, file.path.len() as u64);
    out.extend_from_slice(&file.path);
    out.extend_from_slice(&file.info.id);
    uint(&mut out, file.info.len);
    uint(&mut out, file.info.unit as u64);
    out.extend_from_slice(&file.info.digest);
    uint(&mut out, file.fragments.len() as u64);
    for fragment in &file.fragments {
        uint(&mut out, fragment.pack as u64);
        uint(&mut out, fragment.relative as u64);
        out.extend_from_slice(&fragment.descriptor.encode()[44..53]);
        out.extend_from_slice(&fragment.digest);
    }
    Ok(out)
}
struct Leaf {
    namespace: u8,
    plain: ZeroizingBytes,
    stored: ZeroizingBytes,
    start: u64,
    last: Zeroizing<Vec<u8>>,
    count: usize,
}
fn root_body(
    leaves: &[Leaf],
    packs: usize,
    files: usize,
    used: usize,
    edit: Option<(usize, &[u8])>,
) -> Result<ZeroizingBytes> {
    let upper = 8
        + 30
        + 10
        + 68
        + leaves
            .iter()
            .map(|leaf| 1 + 56 + 20 + leaf.last.len())
            .sum::<usize>();
    if upper > MAX_PLAIN {
        return Err(Error::SecurityLimitExceeded(
            "paged model root budget".into(),
        ));
    }
    let mut out = ZeroizingBytes::new(Vec::with_capacity(MAX_PLAIN));
    out.extend_from_slice(b"RV4MRT01");
    uint(&mut out, files as u64);
    uint(&mut out, packs as u64);
    uint(&mut out, leaves.len() as u64);
    for (index, leaf) in leaves.iter().enumerate() {
        let (start, stored) = match edit {
            Some((changed, bytes)) if changed == index => (PRIVATE_START + used as u64, bytes),
            _ => (leaf.start, leaf.stored.as_slice()),
        };
        out.push(leaf.namespace);
        out.extend_from_slice(&start.to_le_bytes());
        out.extend_from_slice(&(start + REGION).to_le_bytes());
        out.extend_from_slice(&(stored.len() as u64).to_le_bytes());
        out.extend_from_slice(&strong_checksum(stored));
        uint(&mut out, leaf.count as u64);
        uint(&mut out, leaf.last.len() as u64);
        out.extend_from_slice(&leaf.last);
    }
    // Budget the worst edited root: old leaf pending in both banks plus the
    // remaining free tail in both banks. No hidden reserve or allocation tree.
    let staged = edit.map_or(0, |(_, bytes)| bytes.len());
    let free = PRIVATE_BYTES.saturating_sub(used + staged);
    uint(
        &mut out,
        if edit.is_some() { 2 } else { 0 } + if free > 0 { 2 } else { 0 },
    );
    for bank in [0, REGION] {
        if let Some((index, _)) = edit {
            out.push(1);
            out.extend_from_slice(&(bank + leaves[index].start).to_le_bytes());
            out.extend_from_slice(&(leaves[index].stored.len() as u64).to_le_bytes());
        }
        if free > 0 {
            out.push(0);
            out.extend_from_slice(&(bank + PRIVATE_START + (used + staged) as u64).to_le_bytes());
            out.extend_from_slice(&(free as u64).to_le_bytes());
        }
    }
    Ok(out)
}
fn measure(catalogue: &Catalogue, codec: &FrameCodec, auth_bytes: usize) -> Result<Value> {
    let mut leaves = Vec::new();
    let mut records = Vec::new();
    let mut plain_bytes = 10;
    let mut used = 0;
    let finish =
        |records: &mut Vec<&File>, leaves: &mut Vec<Leaf>, used: &mut usize| -> Result<()> {
            if records.is_empty() {
                return Ok(());
            }
            let mut plain = ZeroizingBytes::new(Vec::with_capacity(MAX_PLAIN));
            plain.extend_from_slice(b"RV4MLF01");
            plain.push(1);
            uint(&mut plain, records.len() as u64);
            for file in records.iter() {
                plain.extend_from_slice(&file_record(file)?);
            }
            let (stored, _) = codec.encode(&plain, None)?;
            let leaf = Leaf {
                namespace: 1,
                plain,
                stored,
                start: PRIVATE_START + *used as u64,
                last: records.last().unwrap().path.clone(),
                count: records.len(),
            };
            *used += leaf.stored.len();
            leaves.push(leaf);
            records.clear();
            Ok(())
        };
    for file in &catalogue.files {
        let len = file_record(file)?.len();
        if len + 10 > MAX_LEAF_PLAIN {
            return Ok(
                json!({"inline_geometry_fits":false,"reason":"one file record exceeds leaf model bound; fragment-index overflow required"}),
            );
        }
        if records.len() == LEAF_RECORDS || plain_bytes + len > MAX_LEAF_PLAIN {
            finish(&mut records, &mut leaves, &mut used)?;
            plain_bytes = 10;
        }
        records.push(file);
        plain_bytes += len;
    }
    finish(&mut records, &mut leaves, &mut used)?;
    let file_leaf_count = leaves.len();
    for (group, packs) in catalogue.packs.chunks(LEAF_RECORDS).enumerate() {
        let mut plain = ZeroizingBytes::new(Vec::with_capacity(MAX_PLAIN));
        plain.extend_from_slice(b"RV4MLF01");
        plain.push(2);
        uint(&mut plain, packs.len() as u64);
        for (index, pack) in packs.iter().enumerate() {
            plain.push(2);
            plain.extend_from_slice(&((group * LEAF_RECORDS + index) as u64).to_be_bytes());
            plain.extend_from_slice(&pack.extent.start.to_le_bytes());
            uint(&mut plain, pack.extent.len);
            plain.extend_from_slice(&pack.extent.digest);
            plain.extend_from_slice(&pack.padding_digest);
        }
        let (stored, _) = codec.encode(&plain, None)?;
        let leaf = Leaf {
            namespace: 2,
            plain,
            stored,
            start: PRIVATE_START + used as u64,
            last: Zeroizing::new(
                ((group * LEAF_RECORDS + packs.len() - 1) as u64)
                    .to_be_bytes()
                    .to_vec(),
            ),
            count: packs.len(),
        };
        used += leaf.stored.len();
        leaves.push(leaf);
    }
    let root = root_body(
        &leaves,
        catalogue.packs.len(),
        catalogue.files.len(),
        used,
        None,
    )?;
    let (_, initial_root_required) = codec.encode(&root, None)?;
    let mut largest_staged = 0;
    let mut largest_root = initial_root_required;
    for (index, leaf) in leaves.iter().enumerate() {
        if leaf.namespace != 1 {
            continue;
        }
        let mut changed = leaf.plain.clone();
        // Count is one canonical byte because this model admits at most 32 rows.
        changed[11..15].copy_from_slice(&0o600u32.to_le_bytes());
        let (stored, _) = codec.encode(&changed, None)?;
        largest_staged = largest_staged.max(stored.len());
        let root = root_body(
            &leaves,
            catalogue.packs.len(),
            catalogue.files.len(),
            used,
            Some((index, &stored)),
        )?;
        let (_, required) = codec.encode(&root, None)?;
        largest_root = largest_root.max(required);
        if required <= ROOT_BYTES {
            codec.encode(&root, Some(ROOT_BYTES))?;
        }
    }
    let auth_end = 320 + auth_bytes;
    let eligible = auth_end <= ROOT_START
        && largest_root <= ROOT_BYTES
        && used + largest_staged <= PRIVATE_BYTES;
    Ok(
        json!({"inline_geometry_fits":eligible,"leaf_pages":leaves.len(),"file_leaf_pages":file_leaf_count,"pack_leaf_pages":leaves.len()-file_leaf_count,"max_records_per_leaf":LEAF_RECORDS,"max_leaf_decoded_bytes":MAX_LEAF_PLAIN,"metadata_allocation_quantum":QUANTUM,"live_leaf_bytes_per_bank":used,"largest_staged_leaf_bytes_per_bank":largest_staged,"peak_leaf_bytes_per_bank":used+largest_staged,"private_pool_bytes_per_bank":PRIVATE_BYTES,"private_pool_headroom_bytes":PRIVATE_BYTES as i64-used as i64-largest_staged as i64,"initial_root_required_bytes":initial_root_required,"largest_edited_root_required_bytes":largest_root,"embedded_root_capacity":ROOT_BYTES,"embedded_root_start":ROOT_START,"authenticated_owner_or_mac_bytes":auth_bytes,"publication_auth_end":auth_end,"publication_auth_fits_before_root":auth_end<=ROOT_START,"frame_roundtrips_verified":true,"scope":"one-level file-only cost model; tests one permission change in every leaf, not arbitrary updates or a persisted format"}),
    )
}
pub(super) fn project<S: Storage>(
    files: &mut Files<S>,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<Value> {
    let archive = files.anchor.archive;
    let mode = files.anchor.mode;
    let selected = publication::select(&files.storage, archive, mode, authority)?;
    if selected.commitment != files.anchor.commitment()? {
        return Err(Error::CorruptRecord);
    }
    let auth_bytes = selected.authentication_bytes();
    let codec = FrameCodec::new(archive, mode, key)?;
    let data = Codec::shared_packed(archive, mode, key)?;
    let sealed = std::cell::Cell::new(2 * REGION);
    let mut measured = None;
    let flat = super::cost_model::project_into(
        files,
        |extent, _| {
            sealed.set(extent.start + extent.len);
            Ok(())
        },
        |body| {
            let catalogue = Catalogue::decode(body, &data, sealed.get())?;
            measured = Some(measure(&catalogue, &codec, auth_bytes)?);
            Ok(())
        },
    )?;
    if publication::select(&files.storage, archive, mode, authority)?.commitment
        != selected.commitment
    {
        return Err(Error::CorruptRecord);
    }
    let mut result=measured.unwrap_or_else(||json!({"inline_geometry_fits":false,"reason":"input exceeds existing flat-model admission limit"}));
    result["kind"] = json!("paged_catalogue_cost_model");
    result["flat_catalogue_encoded_bytes"] = flat["catalogue_encoded_bytes"].clone();
    result["payload_allocation_bytes"] = flat["payload_allocation_bytes"].clone();
    result["projected_fresh_and_single_edit_bytes"] = if result["inline_geometry_fits"] == true {
        json!(sealed.get())
    } else {
        Value::Null
    };
    result["source_unchanged"] = json!(true);
    Ok(result)
}
