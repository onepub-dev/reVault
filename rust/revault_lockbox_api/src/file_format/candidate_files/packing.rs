//! Physical packing with independently encoded/authenticated file fragments.
//! Inputs currently share one archive-private access domain; public mixing is forbidden.
use super::*;
use crate::crypto::strong_checksum;
use crate::file_format::allocation_map::Extent;
use crate::page_buffer::{zeroize_bytes, ZeroizingBytes};
const PAD: usize = 65536;
const SLICE_MAGIC: &[u8; 8] = b"RV4SLC02";
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Slice {
    pub descriptor: Descriptor,
    pub start: u32,
    digest: [u8; 32],
    padding_digest: [u8; 32],
}
impl Slice {
    pub fn encode(&self) -> [u8; 144] {
        let mut bytes = [0; 144];
        bytes[..8].copy_from_slice(SLICE_MAGIC);
        bytes[8..72].copy_from_slice(&self.descriptor.encode());
        bytes[72..76].copy_from_slice(&self.start.to_le_bytes());
        bytes[80..112].copy_from_slice(&self.digest);
        bytes[112..].copy_from_slice(&self.padding_digest);
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 144 || &bytes[..8] != SLICE_MAGIC || bytes[76..80] != [0; 4] {
            return Err(Error::CorruptRecord);
        }
        let value = Self {
            descriptor: Descriptor::decode(&bytes[8..72])?,
            start: u32::from_le_bytes(bytes[72..76].try_into().unwrap()),
            digest: bytes[80..112].try_into().unwrap(),
            padding_digest: bytes[112..].try_into().unwrap(),
        };
        value
            .start
            .checked_add(value.descriptor.stored_len() as u32)
            .ok_or(Error::CorruptRecord)?;
        Ok(value)
    }
    pub fn physical(&self, pack: Extent) -> Result<Extent> {
        let start = pack
            .start
            .checked_add(self.start as u64)
            .ok_or(Error::CorruptRecord)?;
        let len = self.descriptor.stored_len() as u64;
        if (self.start as u64)
            .checked_add(len)
            .is_none_or(|end| end > pack.len)
            || start.checked_add(len).is_none()
        {
            return Err(Error::CorruptRecord);
        }
        Ok(Extent {
            start,
            len,
            digest: self.digest,
        })
    }
}
struct PackCoverage {
    extent: Extent,
    padding_digest: [u8; 32],
    logical: usize,
    slices: BTreeMap<u32, u32>,
}
#[derive(Default)]
pub(super) struct Coverage(BTreeMap<u64, PackCoverage>);
impl Coverage {
    pub fn add(&mut self, extent: Extent, slice: &Slice) -> Result<()> {
        slice.physical(extent)?;
        let pack = self.0.entry(extent.start).or_insert_with(|| PackCoverage {
            extent,
            padding_digest: slice.padding_digest,
            logical: 0,
            slices: BTreeMap::new(),
        });
        let end = slice.start + slice.descriptor.stored_len() as u32;
        if pack.extent != extent
            || pack.padding_digest != slice.padding_digest
            || pack
                .slices
                .range(..=slice.start)
                .next_back()
                .is_some_and(|(_, end)| *end > slice.start)
            || pack.slices.range(slice.start..end).next().is_some()
        {
            return Err(Error::CorruptRecord);
        }
        pack.logical = pack
            .logical
            .checked_add(slice.descriptor.logical_len as usize)
            .ok_or(Error::CorruptRecord)?;
        if pack.logical > MAX_LOGICAL {
            return Err(Error::CorruptRecord);
        }
        pack.slices.insert(slice.start, end);
        Ok(())
    }
    pub fn finish(self, storage: &impl Storage, codec: &Codec) -> Result<()> {
        for pack in self.0.into_values() {
            let mut position = 0;
            for (start, end) in pack.slices {
                if start != position {
                    return Err(Error::CorruptRecord);
                }
                position = end;
            }
            let limit = codec.logical_unit(MAX_LOGICAL)?;
            if pack.logical > limit || position as usize > limit + 28 {
                return Err(Error::CorruptRecord);
            }
            let expected = codec.pack_padded_len(position as usize)? as u64;
            if expected != pack.extent.len {
                return Err(Error::CorruptRecord);
            }
            let pad = (expected - position as u64) as usize;
            if pad > 0 {
                let bytes =
                    ZeroizingBytes::new(storage.read_at(pack.extent.start + position as u64, pad)?);
                if bytes.len() != pad || strong_checksum(&bytes) != pack.padding_digest {
                    return Err(Error::CorruptRecord);
                }
                codec.verify_pack_padding(&bytes)?;
            } else if pack.padding_digest != strong_checksum(&[]) {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
}
pub(super) struct Builder<'a> {
    codec: &'a Codec,
    unit: usize,
    logical: usize,
    bytes: ZeroizingBytes,
    members: Vec<Slice>,
}
impl<'a> Builder<'a> {
    pub fn new(codec: &'a Codec, unit: usize) -> Self {
        Self {
            codec,
            unit,
            logical: 0,
            bytes: ZeroizingBytes::new(Vec::with_capacity(unit + PAD)),
            members: Vec::new(),
        }
    }
    pub fn pending(&self) -> usize {
        self.members.len()
    }
    pub fn push<S: Storage>(
        &mut self,
        identity: ([u8; 16], u64, u64),
        bytes: &[u8],
        tx: &mut Transaction<S>,
        entries: &mut Vec<Entry>,
    ) -> Result<()> {
        let (descriptor, stored) = self
            .codec
            .encode(identity.0, identity.1, identity.2, bytes)?;
        self.push_encoded(descriptor, &stored, tx, entries)
    }
    pub fn push_encoded<S: Storage>(
        &mut self,
        descriptor: Descriptor,
        stored: &[u8],
        tx: &mut Transaction<S>,
        entries: &mut Vec<Entry>,
    ) -> Result<()> {
        if stored.len() != descriptor.stored_len()
            || stored.len() > self.unit + 28
            || descriptor.logical_len as usize > self.unit
        {
            return Err(Error::CorruptRecord);
        }
        if self.bytes.len() + stored.len() > self.unit
            || self.logical + descriptor.logical_len as usize > self.unit
        {
            self.flush(tx, entries)?;
        }
        self.logical += descriptor.logical_len as usize;
        self.members.push(Slice {
            descriptor,
            start: self.bytes.len() as u32,
            digest: strong_checksum(stored),
            padding_digest: [0; 32],
        });
        self.bytes.extend_from_slice(stored);
        if self.logical == self.unit || self.bytes.len() >= self.unit {
            self.flush(tx, entries)?;
        }
        Ok(())
    }
    pub fn flush<S: Storage>(
        &mut self,
        tx: &mut Transaction<S>,
        entries: &mut Vec<Entry>,
    ) -> Result<()> {
        if self.bytes.is_empty() {
            return Ok(());
        }
        // Reserve enough capacity up front: padding never reallocates a buffer
        // after private encoded bytes have been written into it.
        let padded = self.codec.pack_padded_len(self.bytes.len())?;
        let padding = self.codec.pack_padding(padded - self.bytes.len())?;
        let padding_digest = strong_checksum(&padding);
        self.bytes.extend_from_slice(&padding);
        let extent = tx.append_encoded_extent(&self.bytes)?;
        for mut slice in self.members.drain(..) {
            slice.padding_digest = padding_digest;
            entries.push(Entry::new(
                CHUNK,
                &chunk_key(slice.descriptor.object, slice.descriptor.ordinal),
                &OwnedRecord::encode(&slice.encode(), &[extent])?,
            )?);
        }
        zeroize_bytes(&mut self.bytes);
        self.bytes.clear();
        self.logical = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_fragment_wire_vector_is_stable() {
        let vector: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/packed_file_fragments_v2.json"
        ))
        .unwrap();
        let mode = FormatMode::new(crate::LockboxFormatOptions {
            encryption: crate::EncryptionMode::None,
            signing: crate::SigningMode::None,
            compression: crate::Compression::None,
            size_padding: crate::SizePadding::Default,
        });
        let codec = Codec::packed(LockboxId::from_bytes([18; 16]), mode, None).unwrap();
        let plain = b"fragment fixture";
        let (descriptor, stored) = codec.encode([7; 16], 0, 0, plain).unwrap();
        let padding = codec.pack_padding(PAD - stored.len()).unwrap();
        let slice = Slice {
            descriptor,
            start: 0,
            digest: strong_checksum(&stored),
            padding_digest: strong_checksum(&padding),
        };
        let info = FileInfo {
            id: [7; 16],
            len: plain.len() as u64,
            unit: 65536,
            digest: Sha256::digest(plain).into(),
        };
        let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        assert_eq!(hex(plain), vector["plain_hex"]);
        assert_eq!(hex(&info.encode()), vector["file_info_hex"]);
        assert_eq!(hex(&slice.encode()), vector["slice_hex"]);
        assert_eq!(Slice::decode(&slice.encode()).unwrap(), slice);
        let mut pack = stored.to_vec();
        pack.extend_from_slice(&padding);
        assert_eq!(
            hex(&strong_checksum(&pack)),
            vector["pack_sha256_domain_hex"]
        );
    }
}
