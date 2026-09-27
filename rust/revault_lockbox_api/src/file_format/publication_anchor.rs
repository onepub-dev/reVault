//! Candidate persistence layer for independent recovery. Not yet activated by
//! the archive reader/writer. Selection authenticates publication, not payload
//! availability; truncation must not silently select an older generation.
use crate::commit_auth::{
    CommitSignature, SIGNATURE_ALGORITHM_ED25519, SIGNATURE_ALGORITHM_ML_DSA_65,
};
use crate::creation_options::FormatMode;
use crate::crypto::strong_checksum;
use crate::storage::Storage;
use crate::{Error, LockboxId, OwnerSigningKeyPair, OwnerSigningPublicKey, Result};
use hmac::{Hmac, Mac};
use sha2::{digest::KeyInit, Sha256};

pub(crate) const SLOT_LEN: usize = 8192;
pub(crate) const FAILURE_REGION: u64 = 65536;
pub(crate) const SLOT_STRIDE: usize = FAILURE_REGION as usize;
pub(crate) const REGION_LEN: usize = 2 * SLOT_STRIDE;
const MAGIC: &[u8; 8] = b"RV4PUB02";
const PREFIX_LEN: usize = 288;
const AUTH_START: usize = 320;
const CHECKSUM_START: usize = SLOT_LEN - 32;
const MAX_ROOT_BYTES: u64 = 65536;
const DOMAIN: &[u8] = b"revault-candidate-publication-v2\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct RootRef {
    pub primary: u64,
    pub mirror: u64,
    pub len: u64,
    pub digest: [u8; 32],
}
impl RootRef {
    fn absent(self) -> bool {
        self == Self::default()
    }
    fn ranges(self, sealed: u64) -> Result<Vec<(u64, u64)>> {
        if self.absent() {
            return Ok(Vec::new());
        }
        if self.len == 0 || self.len > MAX_ROOT_BYTES {
            return Err(Error::CorruptHeader);
        }
        [self.primary, self.mirror]
            .into_iter()
            .map(|offset| {
                let end = offset.checked_add(self.len).ok_or(Error::CorruptHeader)?;
                if offset < REGION_LEN as u64 || end > sealed {
                    return Err(Error::CorruptHeader);
                }
                Ok((offset, end))
            })
            .collect()
    }
    /// Restore only a copy whose peer matches the authenticated stored digest.
    /// Ordinary I/O errors propagate; this does not synthesize missing payload.
    pub(crate) fn repair(&self, storage: &mut impl Storage) -> Result<bool> {
        self.ranges(storage.len()?)?;
        if self.absent() {
            return Err(Error::CorruptRecord);
        }
        let primary = zeroize::Zeroizing::new(storage.read_at(self.primary, self.len as usize)?);
        let mirror = zeroize::Zeroizing::new(storage.read_at(self.mirror, self.len as usize)?);
        match (
            strong_checksum(&primary) == self.digest,
            strong_checksum(&mirror) == self.digest,
        ) {
            (true, true) => Ok(false),
            (true, false) => {
                storage.write_at(self.mirror, &primary)?;
                Ok(true)
            }
            (false, true) => {
                storage.write_at(self.primary, &mirror)?;
                Ok(true)
            }
            (false, false) => Err(Error::CorruptRecord),
        }
    }
    /// Recover one root copy without treating an unreadable source as permission
    /// to roll publication backward. A damaged copy may use its verified mirror.
    pub(crate) fn read_verified(self, storage: &impl Storage) -> Result<Vec<u8>> {
        self.ranges(u64::MAX)?;
        if self.absent() {
            return Err(Error::CorruptRecord);
        }
        for offset in [self.primary, self.mirror] {
            if offset.checked_add(self.len).ok_or(Error::CorruptRecord)? > storage.len()? {
                continue;
            }
            let bytes = storage.read_at(offset, self.len as usize)?;
            if strong_checksum(&bytes) == self.digest {
                return Ok(bytes);
            }
        }
        Err(Error::CorruptRecord)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Anchor {
    pub archive: LockboxId,
    pub generation: u64,
    pub mode: FormatMode,
    pub sealed_len: u64,
    pub object_root: [u8; 32],
    pub previous: [u8; 32],
    pub index: RootRef,
    pub allocation: RootRef,
    pub keys: RootRef,
}
impl Anchor {
    fn validate(&self) -> Result<()> {
        FormatMode::parse(self.mode.0)?;
        if self.generation == 0
            || self.sealed_len < REGION_LEN as u64
            || self.index.absent()
            || ((self.generation == 1) != (self.previous == [0; 32]))
        {
            return Err(Error::CorruptHeader);
        }
        let mut ranges = Vec::new();
        for root in [self.index, self.allocation, self.keys] {
            ranges.extend(root.ranges(self.sealed_len)?);
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(Error::CorruptHeader);
        }
        Ok(())
    }
    fn prefix(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut out = Vec::with_capacity(PREFIX_LEN);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&self.mode.0.to_le_bytes());
        out.extend_from_slice(&(SLOT_LEN as u32).to_le_bytes());
        out.extend_from_slice(&self.generation.to_le_bytes());
        out.extend_from_slice(self.archive.as_bytes());
        out.extend_from_slice(&self.sealed_len.to_le_bytes());
        // Object counts remain in private index metadata, not the public anchor.
        out.extend_from_slice(&[0; 8]);
        out.extend_from_slice(&self.object_root);
        out.extend_from_slice(&self.previous);
        for root in [self.index, self.allocation, self.keys] {
            out.extend_from_slice(&root.primary.to_le_bytes());
            out.extend_from_slice(&root.mirror.to_le_bytes());
            out.extend_from_slice(&root.len.to_le_bytes());
            out.extend_from_slice(&root.digest);
        }
        debug_assert_eq!(out.len(), PREFIX_LEN);
        Ok(out)
    }
    pub(crate) fn commitment(&self) -> Result<[u8; 32]> {
        Ok(strong_checksum(&message(&self.prefix()?)))
    }
    fn verify_dependencies(&self, storage: &impl Storage) -> Result<()> {
        self.validate()?;
        if storage.len()? < self.sealed_len {
            return Err(Error::Truncated);
        }
        for root in [self.index, self.allocation, self.keys] {
            if root.absent() {
                continue;
            }
            for offset in [root.primary, root.mirror] {
                if strong_checksum(&storage.read_at(offset, root.len as usize)?) != root.digest {
                    return Err(Error::CorruptRecord);
                }
            }
        }
        Ok(())
    }
}
fn message(prefix: &[u8]) -> Vec<u8> {
    [DOMAIN, prefix].concat()
}
fn symmetric_mac(key: &[u8], message: &[u8]) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts arbitrary key lengths");
    mac.update(b"revault-candidate-publication-mac-v2\0");
    mac.update(message);
    mac
}

/// The expected mode, archive and owner come from the caller's established trust
/// context. Never derive an owner-write policy from an untrusted slot's flags.
pub(crate) enum Authority<'a> {
    Owner(&'a OwnerSigningPublicKey),
    Symmetric(&'a [u8]),
    Checksum,
}
impl Authority<'_> {
    fn accepts(&self, mode: FormatMode) -> bool {
        matches!(
            (self, mode.signed(), mode.plaintext()),
            (Self::Owner(_), true, _)
                | (Self::Symmetric(_), false, false)
                | (Self::Checksum, false, true)
        )
    }
}

fn encode(
    anchor: &Anchor,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> Result<Vec<u8>> {
    if !authority.accepts(anchor.mode) {
        return Err(Error::CorruptHeader);
    }
    let prefix = anchor.prefix()?;
    let message = message(&prefix);
    let mut auth = Vec::new();
    match authority {
        Authority::Owner(owner) => {
            let signatures = signer.ok_or(Error::CorruptRecord)?.sign(&message);
            owner.verify_publication_signatures(&message, &signatures)?;
            auth.extend_from_slice(&2u16.to_le_bytes());
            for signature in signatures {
                auth.extend_from_slice(&signature.algorithm.to_le_bytes());
                auth.extend_from_slice(&(signature.public_key.len() as u32).to_le_bytes());
                auth.extend_from_slice(&(signature.signature.len() as u32).to_le_bytes());
                auth.extend_from_slice(&signature.public_key);
                auth.extend_from_slice(&signature.signature);
            }
        }
        Authority::Symmetric(key) => {
            if signer.is_some() {
                return Err(Error::CorruptRecord);
            }
            auth.extend_from_slice(&symmetric_mac(key, &message).finalize().into_bytes());
        }
        Authority::Checksum => {
            if signer.is_some() {
                return Err(Error::CorruptRecord);
            }
        }
    }
    if auth.len() > CHECKSUM_START - AUTH_START {
        return Err(Error::CorruptRecord);
    }
    let mut out = vec![0; SLOT_LEN];
    out[..PREFIX_LEN].copy_from_slice(&prefix);
    out[PREFIX_LEN..PREFIX_LEN + 4].copy_from_slice(&(auth.len() as u32).to_le_bytes());
    out[AUTH_START..AUTH_START + auth.len()].copy_from_slice(&auth);
    let checksum = strong_checksum(&out[..CHECKSUM_START]);
    out[CHECKSUM_START..].copy_from_slice(&checksum);
    Ok(out)
}

// Structural inspection is not authentication. Only credential bootstrap may
// use it to locate a bounded public key directory; ordinary selection verifies
// the owner/MAC before returning any publication authority.
fn parse_untrusted(slot: &[u8], archive: LockboxId, mode: FormatMode) -> Result<Anchor> {
    if slot.len() != SLOT_LEN
        || &slot[..8] != MAGIC
        || slot[8..10] != 2u16.to_le_bytes()
        || slot[10..12] != mode.0.to_le_bytes()
        || slot[12..16] != (SLOT_LEN as u32).to_le_bytes()
        || slot[48..56].iter().any(|b| *b != 0)
        || slot[292..AUTH_START].iter().any(|b| *b != 0)
        || strong_checksum(&slot[..CHECKSUM_START]) != slot[CHECKSUM_START..]
    {
        return Err(Error::CorruptHeader);
    }
    let len = u32::from_le_bytes(slot[288..292].try_into().unwrap()) as usize;
    if len > CHECKSUM_START - AUTH_START
        || slot[AUTH_START + len..CHECKSUM_START]
            .iter()
            .any(|b| *b != 0)
    {
        return Err(Error::CorruptHeader);
    }
    let mut reader = Reader::new(&slot[16..PREFIX_LEN]);
    let generation = reader.u64()?;
    let found_archive = LockboxId::from_bytes(reader.take(16)?.try_into().unwrap());
    if found_archive != archive {
        return Err(Error::CorruptHeader);
    }
    let sealed_len = reader.u64()?;
    reader.take(8)?; // Reserved; the fixed prefix check above requires zero.
    let anchor = Anchor {
        generation,
        archive,
        mode,
        sealed_len,
        object_root: reader.take(32)?.try_into().unwrap(),
        previous: reader.take(32)?.try_into().unwrap(),
        index: reader.root()?,
        allocation: reader.root()?,
        keys: reader.root()?,
    };
    reader.done()?;
    anchor.validate()?;
    Ok(anchor)
}
fn decode(
    slot: &[u8],
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Result<Anchor> {
    if !authority.accepts(mode) {
        return Err(Error::CorruptHeader);
    }
    let anchor = parse_untrusted(slot, archive, mode)?;
    let len = u32::from_le_bytes(slot[288..292].try_into().unwrap()) as usize;
    let auth = &slot[AUTH_START..AUTH_START + len];
    let message = message(&slot[..PREFIX_LEN]);
    match authority {
        Authority::Owner(owner) => {
            let mut reader = Reader::new(auth);
            if reader.u16()? != 2 {
                return Err(Error::CorruptHeader);
            }
            let mut signatures = Vec::with_capacity(2);
            for expected in [SIGNATURE_ALGORITHM_ED25519, SIGNATURE_ALGORITHM_ML_DSA_65] {
                let algorithm = reader.u16()?;
                let key_len = reader.u32()? as usize;
                let signature_len = reader.u32()? as usize;
                if algorithm != expected || key_len > 4096 || signature_len > 4096 {
                    return Err(Error::CorruptHeader);
                }
                signatures.push(CommitSignature {
                    algorithm,
                    public_key: reader.take(key_len)?.to_vec(),
                    signature: reader.take(signature_len)?.to_vec(),
                });
            }
            reader.done()?;
            owner.verify_publication_signatures(&message, &signatures)?;
        }
        Authority::Symmetric(key) => {
            symmetric_mac(key, &message)
                .verify_slice(auth)
                .map_err(|_| Error::CorruptHeader)?;
        }
        Authority::Checksum => {
            if !auth.is_empty() {
                return Err(Error::CorruptHeader);
            }
        }
    }
    Ok(anchor)
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.position.checked_add(len).ok_or(Error::CorruptHeader)?;
        let value = self.bytes.get(self.position..end).ok_or(Error::Truncated)?;
        self.position = end;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn root(&mut self) -> Result<RootRef> {
        Ok(RootRef {
            primary: self.u64()?,
            mirror: self.u64()?,
            len: self.u64()?,
            digest: self.take(32)?.try_into().unwrap(),
        })
    }
    fn done(&self) -> Result<()> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(Error::CorruptHeader)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Selection {
    pub anchor: Anchor,
    pub commitment: [u8; 32],
    slot: usize,
    copies: u8,
    encoded: Vec<u8>,
}
/// Success token from a synchronized publication/mirror operation. Merely reading
/// matching copies cannot produce this token: their last sync may have failed.
#[derive(Debug)]
pub(crate) struct Published {
    pub anchor: Anchor,
    pub commitment: [u8; 32],
}

pub(crate) fn select(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
) -> Result<Selection> {
    let len = storage.len()?;
    let mut candidates = Vec::with_capacity(2);
    for slot in 0..2 {
        let offset = (slot * SLOT_STRIDE) as u64;
        if offset + SLOT_LEN as u64 > len {
            continue;
        }
        // An I/O error is not evidence that the possibly newer slot is corrupt.
        let encoded = storage.read_at(offset, SLOT_LEN)?;
        if let Ok(anchor) = decode(&encoded, archive, mode, authority) {
            candidates.push(Selection {
                commitment: anchor.commitment()?,
                anchor,
                slot,
                copies: 1 << slot,
                encoded,
            });
        }
    }
    candidates.sort_by_key(|candidate| candidate.anchor.generation);
    let mut best = candidates.pop().ok_or(Error::CorruptHeader)?;
    if let Some(old) = candidates.pop() {
        if old.anchor.generation == best.anchor.generation {
            if old.commitment != best.commitment {
                return Err(Error::CorruptHeader);
            }
            best.copies |= old.copies;
        } else if old.anchor.generation.checked_add(1) != Some(best.anchor.generation)
            || best.anchor.previous != old.commitment
        {
            return Err(Error::CorruptHeader);
        }
    }
    Ok(best)
}

pub(crate) fn ensure_mirrored(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    expected: [u8; 32],
) -> Result<Published> {
    let selected = select(storage, archive, mode, authority)?;
    if selected.commitment != expected {
        return Err(Error::CorruptHeader);
    }
    selected.anchor.verify_dependencies(storage)?;
    // Required even when both copies are readable after a failed final sync.
    storage.sync()?;
    if selected.copies != 3 {
        storage.write_at(
            ((1 - selected.slot) * SLOT_STRIDE) as u64,
            &selected.encoded,
        )?;
        storage.sync()?;
    }
    Ok(Published {
        anchor: selected.anchor,
        commitment: selected.commitment,
    })
}

pub(crate) fn publish(
    storage: &mut impl Storage,
    next: &Anchor,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    expected: Option<[u8; 32]>,
) -> Result<Published> {
    next.verify_dependencies(storage)?;
    if storage.len()? != next.sealed_len {
        return Err(Error::CorruptHeader);
    }
    let current = match expected {
        Some(expected) => {
            let current = select(storage, next.archive, next.mode, authority)?;
            if current.commitment != expected
                || next.previous != expected
                || current.anchor.generation.checked_add(1) != Some(next.generation)
            {
                return Err(Error::CorruptHeader);
            }
            Some(current)
        }
        None => {
            if next.generation != 1 || storage.read_at(0, REGION_LEN)?.iter().any(|b| *b != 0) {
                return Err(Error::CorruptHeader);
            }
            None
        }
    };
    // Validate all input/authority before mutating a publication slot.
    let encoded = encode(next, authority, signer)?;
    if let Some(current) = &current {
        if current.copies != 3 {
            ensure_mirrored(
                storage,
                next.archive,
                next.mode,
                authority,
                current.commitment,
            )?;
        }
    }
    storage.sync()?; // Prepared dependencies must be durable before publication.
    let first = current.map_or(0, |current| 1 - current.slot);
    storage.write_at((first * SLOT_STRIDE) as u64, &encoded)?;
    storage.sync()?;
    storage.write_at(((1 - first) * SLOT_STRIDE) as u64, &encoded)?;
    storage.sync()?;
    Ok(Published {
        anchor: next.clone(),
        commitment: next.commitment()?,
    })
}

/// Publish a verified relocation into a separate, unpublished replacement.
/// The caller must preserve logical membership and hold the source's writer lock
/// through verification and installation. Generation and predecessor are retained
/// across replacement; this must not reset an existing archive to generation one.
pub(crate) fn publish_relocation(
    source: &impl Storage,
    destination: &mut impl Storage,
    next: &Anchor,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> Result<Published> {
    let current = select(source, next.archive, next.mode, authority)?;
    if next.previous != current.commitment
        || current.anchor.generation.checked_add(1) != Some(next.generation)
        || destination.len()? != next.sealed_len
        || destination.read_at(0, REGION_LEN)?.iter().any(|b| *b != 0)
    {
        return Err(Error::CorruptHeader);
    }
    next.verify_dependencies(destination)?;
    let encoded = encode(next, authority, signer)?;
    destination.sync()?;
    destination.write_at(0, &encoded)?;
    destination.sync()?;
    destination.write_at(SLOT_STRIDE as u64, &encoded)?;
    destination.sync()?;
    Ok(Published {
        anchor: next.clone(),
        commitment: next.commitment()?,
    })
}

#[cfg(test)]
#[path = "publication_anchor_tests.rs"]
mod tests;

pub(crate) mod bootstrap;
