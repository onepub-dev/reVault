//! Credential-bootstrap experiment using existing wrapping and publication crypto.
//! This is not yet wired into the candidate allocator/public API or inline control
//! placement. Caller retains a stable source snapshot/read lock for the operation.
use super::*;
use crate::file_format::key_directory::{encode_key_directory, read_key_directory_backup};
use crate::key_slot::KeySlot;
use crate::page_buffer::ZeroizingBytes;
use crate::{ContactKeyPair, SecretString, SecretVec};
use std::collections::BTreeSet;

pub(crate) const INLINE_BYTES: usize = 4096;
const MAX_INLINE_SLOTS: usize = 48;

#[derive(Clone, Copy)]
pub(crate) enum Credential<'a> {
    Password(&'a SecretString),
    Contact(&'a ContactKeyPair),
}
pub(crate) struct Opened {
    pub anchor: Anchor,
    pub key: SecretVec,
}

/// Public slots contain existing encrypted key wrappers and IDs, never private
/// labels or owner signing keys. Oversized directories need authenticated overflow
/// in the eventual layout; this component explicitly refuses them for now.
pub(crate) fn directory(archive: LockboxId, generation: u64, slots: &[KeySlot]) -> Result<Vec<u8>> {
    if generation == 0 || slots.is_empty() || slots.len() > MAX_INLINE_SLOTS {
        return Err(Error::InvalidInput(
            "invalid inline bootstrap slot count/generation".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    if slots
        .iter()
        .any(|slot| slot.id() == 0 || !ids.insert(slot.id()))
    {
        return Err(Error::InvalidInput(
            "duplicate or zero bootstrap slot ID".into(),
        ));
    }
    let mut encoded = encode_key_directory(slots, archive, generation, 0)?;
    if encoded.len() > INLINE_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "bootstrap directory requires authenticated overflow".into(),
        ));
    }
    encoded.resize(INLINE_BYTES, 0);
    Ok(encoded)
}
fn read_directory(storage: &impl Storage, anchor: &Anchor) -> Result<Vec<KeySlot>> {
    read_directory_in(storage, anchor, Layout::Separated)
}
pub(super) fn read_directory_in(
    storage: &impl Storage,
    anchor: &Anchor,
    layout: Layout,
) -> Result<Vec<KeySlot>> {
    if anchor.keys.len != INLINE_BYTES as u64 {
        return Err(Error::CorruptHeader);
    }
    let encoded = match layout {
        Layout::Separated => anchor.keys.read_verified(storage)?,
        Layout::Shared => super::shared::read_root(storage, anchor, RootRole::PublicKeys)?,
    };
    let decoded = read_key_directory_backup(&encoded)?;
    if decoded.lockbox_id != anchor.archive
        || decoded.generation == 0
        || decoded.generation > anchor.generation
        || decoded.copy_index != 0
        || directory(decoded.lockbox_id, decoded.generation, &decoded.slots)? != encoded
    {
        return Err(Error::CorruptHeader);
    }
    Ok(decoded.slots)
}

/// Unauthenticated structural selection is only used before a symmetric key is
/// available. Never fall back to a lower checksummed generation on unwrap/MAC
/// failure: it may be a genuine key rotation the old credential must not bypass.
/// A forged higher checksummed slot can therefore deny bootstrap, just as erasing
/// a slot can deny access. Explicit-key/owner-authenticated selection remains separate.
fn unsigned_candidate(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    layout: Layout,
) -> Result<Anchor> {
    let mut candidates = Vec::with_capacity(2);
    let len = storage.len()?;
    for slot in 0..2 {
        let offset = (slot * SLOT_STRIDE) as u64;
        if offset + SLOT_LEN as u64 > len {
            continue;
        }
        let bytes = storage.read_at(offset, SLOT_LEN)?;
        if let Ok(anchor) = parse_untrusted_in(&bytes, archive, mode, layout) {
            if u32::from_le_bytes(bytes[288..292].try_into().unwrap()) != 32 {
                continue;
            }
            candidates.push(anchor);
        }
    }
    candidates.sort_by_key(|candidate| candidate.generation);
    let best = candidates.pop().ok_or(Error::CorruptHeader)?;
    if let Some(old) = candidates.pop() {
        if (best.generation == old.generation
            && best.commitment_in(layout)? != old.commitment_in(layout)?)
            || (best.generation != old.generation
                && (old.generation.checked_add(1) != Some(best.generation)
                    || best.previous != old.commitment_in(layout)?))
        {
            return Err(Error::CorruptHeader);
        }
    }
    Ok(best)
}

pub(crate) fn open(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    owner: Option<&OwnerSigningPublicKey>,
    credential: Credential<'_>,
    requested_slot: Option<u64>,
) -> Result<Opened> {
    open_in(
        storage,
        archive,
        mode,
        owner,
        credential,
        requested_slot,
        Layout::Separated,
    )
}
pub(super) fn open_in(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    owner: Option<&OwnerSigningPublicKey>,
    credential: Credential<'_>,
    requested_slot: Option<u64>,
    layout: Layout,
) -> Result<Opened> {
    if mode.plaintext() {
        return Err(Error::InvalidInput(
            "credential bootstrap requires encrypted mode".into(),
        ));
    }
    let candidate = if mode.signed() {
        let owner = owner.ok_or_else(|| {
            Error::InvalidInput("signed bootstrap requires the established owner".into())
        })?;
        select_in(storage, archive, mode, &Authority::Owner(owner), layout)?.anchor
    } else {
        if owner.is_some() {
            return Err(Error::InvalidInput(
                "unsigned bootstrap cannot claim owner authorization".into(),
            ));
        }
        unsigned_candidate(storage, archive, mode, layout)?
    };
    let expected = candidate.commitment_in(layout)?;
    let slots = read_directory_in(storage, &candidate, layout)?;
    for slot in &slots {
        if requested_slot.is_some_and(|id| id != slot.id()) {
            continue;
        }
        let result = match (slot, credential) {
            (KeySlot::Password { .. }, Credential::Password(password)) => {
                slot.try_password(password)
            }
            (KeySlot::HybridContact { .. }, Credential::Contact(contact)) => {
                slot.try_contact(contact)
            }
            _ => continue,
        };
        let decoded = match result {
            Ok(bytes) => ZeroizingBytes::new(bytes),
            Err(_) => continue,
        };
        if decoded.len() != 32 {
            continue;
        }
        let authenticated = if let Some(owner) = owner {
            select_in(storage, archive, mode, &Authority::Owner(owner), layout)?
        } else {
            select_in(
                storage,
                archive,
                mode,
                &Authority::Symmetric(&decoded),
                layout,
            )?
        };
        if authenticated.commitment != expected {
            return Err(Error::CorruptHeader);
        }
        return Ok(Opened {
            anchor: authenticated.anchor,
            key: SecretVec::try_from_slice(&decoded)?,
        });
    }
    Err(Error::InvalidKey)
}

#[cfg(test)]
mod tests;
