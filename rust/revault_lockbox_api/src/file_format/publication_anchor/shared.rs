//! Shared-control placement/authority component. No public writer uses this
//! profile. Inline metadata and public key slots have distinct permitted ranges;
//! they cannot alias publication or preparation. Overflow and allocation/retirement
//! integration remain separate requirements before activating a whole archive.
use super::*;

const PRIVATE_START: u64 = 16384;
const PRIVATE_BYTES: usize = 49152;
const KEYS_START: u64 = 12288;

fn commitment(anchor: &Anchor) -> Result<[u8; 32]> {
    Ok(strong_checksum(&message(
        &anchor.prefix_in(Layout::Shared)?,
    )))
}
pub(super) fn read_root(
    storage: &impl Storage,
    anchor: &Anchor,
    role: RootRole,
) -> Result<Vec<u8>> {
    anchor.validate_in(Layout::Shared)?;
    let root = match role {
        RootRole::Private => anchor.index,
        RootRole::Allocation => anchor.allocation,
        RootRole::PublicKeys => anchor.keys,
    };
    Layout::Shared.ranges(root, anchor.sealed_len, role)?;
    if root.absent() {
        return Err(Error::CorruptRecord);
    }
    for offset in [root.primary, root.mirror] {
        if offset.checked_add(root.len).ok_or(Error::CorruptRecord)? > storage.len()? {
            continue;
        }
        let bytes = storage.read_at(offset, root.len as usize)?;
        if strong_checksum(&bytes) == root.digest {
            return Ok(bytes);
        }
    }
    Err(Error::CorruptRecord)
}

#[cfg(test)]
mod tests;

mod catalogue;

/// Publish a freshly staged comparison image. The file-image builder validates
/// all payload membership/bytes first and owns cleanup of the empty destination.
/// This function does not perform updates, install a path, or authorize retirement.
#[allow(clippy::too_many_arguments)]
pub(crate) fn initialize(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    body: &[u8],
    slots: &[crate::key_slot::KeySlot],
) -> Result<Anchor> {
    if storage.len()? < REGION_LEN as u64
        || storage
            .read_at(0, REGION_LEN)?
            .iter()
            .any(|byte| *byte != 0)
    {
        return Err(Error::InvalidInput(
            "shared image requires an empty control prefix".into(),
        ));
    }
    if let Authority::Symmetric(authority_key) = authority {
        if key != Some(*authority_key) {
            return Err(Error::InvalidKey);
        }
    }
    if mode.plaintext() && !slots.is_empty() {
        return Err(Error::InvalidInput(
            "plaintext image cannot contain decryption slots".into(),
        ));
    }
    let private = catalogue::Codec::new(archive, mode, key)?.encode(body)?;
    let public = if slots.is_empty() {
        None
    } else {
        Some(super::bootstrap::directory(archive, 1, slots)?)
    };
    let index = RootRef {
        primary: PRIVATE_START,
        mirror: FAILURE_REGION + PRIVATE_START,
        len: private.len() as u64,
        digest: strong_checksum(&private),
    };
    let keys = public.as_ref().map_or(RootRef::default(), |bytes| RootRef {
        primary: KEYS_START,
        mirror: FAILURE_REGION + KEYS_START,
        len: bytes.len() as u64,
        digest: strong_checksum(bytes),
    });
    let anchor = Anchor {
        archive,
        generation: 1,
        mode,
        sealed_len: storage.len()?,
        object_root: index.digest,
        previous: [0; 32],
        index,
        allocation: RootRef::default(),
        keys,
    };
    anchor.validate_in(Layout::Shared)?;
    let base = commitment(&anchor)?;
    let stub =
        crate::file_format::preparation_journal::compact::initial_stub(archive, mode, key, base)?;
    let encoded = encode_in(&anchor, authority, signer, Layout::Shared)?;
    for bank in [0, FAILURE_REGION] {
        storage.write_at(bank + PRIVATE_START, &private)?;
        if let Some(public) = &public {
            storage.write_at(bank + KEYS_START, public)?;
        }
        storage.write_at(bank + 8192, &stub)?;
    }
    // Read back both dependencies before exposing either publication copy.
    for root in [index, keys] {
        if root.absent() {
            continue;
        }
        for offset in [root.primary, root.mirror] {
            if strong_checksum(&storage.read_at(offset, root.len as usize)?) != root.digest {
                return Err(Error::CorruptRecord);
            }
        }
    }
    crate::file_format::preparation_journal::compact::validate_initial_idle(
        storage, archive, mode, key, base,
    )?;
    storage.sync()?;
    storage.write_at(0, &encoded)?;
    storage.sync()?;
    storage.write_at(FAILURE_REGION, &encoded)?;
    storage.sync()?;
    Ok(anchor)
}
pub(crate) fn open_private(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<(Anchor, crate::page_buffer::ZeroizingBytes)> {
    private_in(storage, archive, mode, authority, key, true)
}
/// Explicit read-only recovery trusts selected publication/private membership,
/// not unrelated preparation, public wrappers, payload availability or padding.
pub(crate) fn salvage_private(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<(Anchor, crate::page_buffer::ZeroizingBytes)> {
    private_in(storage, archive, mode, authority, key, false)
}
fn private_in(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    strict: bool,
) -> Result<(Anchor, crate::page_buffer::ZeroizingBytes)> {
    let selected = select_in(storage, archive, mode, authority, Layout::Shared)?;
    let anchor = selected.anchor;
    validate_fresh_shape(&anchor)?;
    if strict && storage.len()? != anchor.sealed_len {
        return Err(Error::CorruptRecord);
    }
    if strict {
        if !anchor.keys.absent() {
            super::bootstrap::read_directory_in(storage, &anchor, Layout::Shared)?;
        }
        crate::file_format::preparation_journal::compact::validate_initial_idle(
            storage,
            archive,
            mode,
            key,
            selected.commitment,
        )?;
    }
    let stored =
        crate::page_buffer::ZeroizingBytes::new(read_root(storage, &anchor, RootRole::Private)?);
    let body = catalogue::Codec::new(archive, mode, key)?.decode(&stored)?;
    Ok((anchor, body))
}

pub(crate) fn credential_open(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    owner: Option<&OwnerSigningPublicKey>,
    credential: super::bootstrap::Credential<'_>,
    slot: Option<u64>,
) -> Result<super::bootstrap::Opened> {
    super::bootstrap::open_in(
        storage,
        archive,
        mode,
        owner,
        credential,
        slot,
        Layout::Shared,
    )
}

/// The fresh file catalogue partitions every byte after the control prefix as
/// payload. External metadata/key roots would alias that graph; general shared
/// root placement alone is not sufficient ownership validation for this adapter.
fn validate_fresh_shape(anchor: &Anchor) -> Result<()> {
    anchor.validate_in(Layout::Shared)?;
    if anchor.generation != 1
        || anchor.index.len != PRIVATE_BYTES as u64
        || anchor.object_root != anchor.index.digest
        || !anchor.allocation.absent()
        || anchor.index.primary >= REGION_LEN as u64
        || anchor.index.mirror >= REGION_LEN as u64
        || (!anchor.keys.absent()
            && (anchor.keys.primary >= REGION_LEN as u64
                || anchor.keys.mirror >= REGION_LEN as u64))
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}
