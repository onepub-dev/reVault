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
