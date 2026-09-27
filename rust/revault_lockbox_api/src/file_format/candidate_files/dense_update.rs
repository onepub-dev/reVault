//! Bounded file-metadata COW experiment. Caller holds exclusive archive ownership
//! for the entire operation and any recovery. No public API, payload mutation,
//! catalogue overflow, journal-overflow rotation or path installation is exposed.
use super::dense_catalogue::Catalogue;
use super::dense_image::Image;
use super::*;
use crate::crypto::strong_checksum;
use crate::file_format::preparation_journal::compact::session::InlineSession;
use crate::file_format::preparation_journal::{Reservation, FREE, PENDING};
use crate::file_format::publication_anchor::{shared, RootRef, FAILURE_REGION, REGION_LEN};
use shared::ownership::{Span, Vacant, VacantKind};
const PRIVATE_BYTES: u64 = 49152;
const PRIVATE_START: u64 = 16384;
fn zero(storage: &mut impl Storage, span: Span) -> Result<()> {
    let mut position = span.start;
    let end = position.checked_add(span.len).ok_or(Error::CorruptRecord)?;
    let bytes = [0u8; 65536];
    while position < end {
        let n = (end - position).min(bytes.len() as u64) as usize;
        storage.write_at(position, &bytes[..n])?;
        position += n as u64;
    }
    Ok(())
}
fn zero_tail_and_truncate(storage: &mut impl Storage, sealed: u64) -> Result<()> {
    let len = storage.len()?;
    if len < sealed {
        return Err(Error::Truncated);
    }
    if len > sealed {
        zero(
            storage,
            Span {
                start: sealed,
                len: len - sealed,
            },
        )?;
        storage.sync()?;
        storage.truncate(sealed)?;
        storage.sync()?;
    }
    Ok(())
}
pub(super) fn recover(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<Anchor> {
    let (anchor, body) = shared::snapshot(storage, archive, mode, authority, key)?;
    let codec = Codec::shared_packed(archive, mode, key)?;
    let catalogue = Catalogue::decode(&body, &codec, anchor.sealed_len)?;
    let graph = catalogue.graph(&anchor)?;
    let commit = shared::commitment(&anchor)?;
    let mut journal = InlineSession::open(storage, archive, mode, key)?;
    if !journal.active() {
        journal.require_idle(commit)?;
        if storage.len()? != anchor.sealed_len {
            return Err(Error::CorruptRecord);
        }
        graph.verify_reclaimed(storage)?;
        shared::ensure_mirrored(storage, archive, mode, authority, commit)?;
        journal.mirror(storage)?;
        return Ok(anchor);
    }
    if journal.base() != commit && journal.base() != anchor.previous {
        return Err(Error::CorruptRecord);
    }
    // Validate all cleanup authority before the first write. A losing transaction
    // may clean only its selected old graph's reservations, never live bytes.
    let ranges = if journal.base() == commit {
        graph.validate_reservations(journal.reservations())?
    } else {
        graph.pending()
    };
    if storage.len()? < anchor.sealed_len {
        return Err(Error::Truncated);
    }
    // This produces a durability barrier, rather than trusting two readable slots.
    shared::ensure_mirrored(storage, archive, mode, authority, commit)?;
    for span in ranges {
        zero(storage, span)?;
    }
    storage.sync()?;
    // Active preparation owns the uncommitted append tail. Erase before truncating.
    zero_tail_and_truncate(storage, anchor.sealed_len)?;
    graph.verify_reclaimed(storage)?;
    journal.finish(storage, commit)?;
    Ok(anchor)
}
/// Select a vacant private slot. Each external slot starts in its own 64 KiB
/// failure region; leftover alignment/tail bytes remain explicit zeroed free space.
fn take_slot(vacant: &mut Vec<Vacant>, wanted: Option<u64>, sealed: &mut u64) -> Result<u64> {
    let found = vacant.iter().enumerate().find_map(|(index, entry)| {
        let start = if let Some(wanted) = wanted {
            wanted
        } else {
            if entry.span.start < REGION_LEN as u64 {
                return None;
            }
            entry.span.start.checked_add(FAILURE_REGION - 1)? / FAILURE_REGION * FAILURE_REGION
        };
        (start >= entry.span.start
            && start.checked_add(PRIVATE_BYTES)? <= entry.span.start.checked_add(entry.span.len)?)
        .then_some((index, start))
    });
    if let Some((index, start)) = found {
        let old = vacant.remove(index);
        let end = old.span.start + old.span.len;
        if start > old.span.start {
            vacant.push(Vacant {
                span: Span {
                    start: old.span.start,
                    len: start - old.span.start,
                },
                kind: old.kind,
            });
        }
        if start + PRIVATE_BYTES < end {
            vacant.push(Vacant {
                span: Span {
                    start: start + PRIVATE_BYTES,
                    len: end - start - PRIVATE_BYTES,
                },
                kind: old.kind,
            });
        }
        return Ok(start);
    }
    if wanted.is_some() {
        return Err(Error::CorruptRecord);
    }
    let start = sealed
        .checked_add(FAILURE_REGION - 1)
        .ok_or(Error::CorruptRecord)?
        / FAILURE_REGION
        * FAILURE_REGION;
    if start > *sealed {
        vacant.push(Vacant {
            span: Span {
                start: *sealed,
                len: start - *sealed,
            },
            kind: VacantKind::Free,
        });
    }
    *sealed = start
        .checked_add(FAILURE_REGION)
        .ok_or(Error::CorruptRecord)?;
    vacant.push(Vacant {
        span: Span {
            start: start + PRIVATE_BYTES,
            len: FAILURE_REGION - PRIVATE_BYTES,
        },
        kind: VacantKind::Free,
    });
    Ok(start)
}
/// Rename one file and optionally replace its permission bits. Identities,
/// fragment AEAD bindings, payload bytes and wrapped keys remain unchanged.
#[allow(clippy::too_many_arguments)]
pub(super) fn edit(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    from: &[u8],
    to: &[u8],
    permissions: Option<u32>,
) -> Result<bool> {
    if !valid_path(from) || !valid_path(to) || permissions.is_some_and(|bits| bits & !0o7777 != 0) {
        return Err(Error::InvalidInput("invalid file metadata edit".into()));
    }
    recover(storage, archive, mode, authority, key)?;
    // Retain normal-open padding checks and eager signed plaintext, and verify
    // protected payloads before updating their authenticated membership as well.
    Image::open(
        allocation::compaction::View(storage),
        archive,
        mode,
        authority,
        key,
    )?
    .verify_all()?;
    let (anchor, body) = shared::snapshot(storage, archive, mode, authority, key)?;
    let codec = Codec::shared_packed(archive, mode, key)?;
    let mut catalogue = Catalogue::decode(&body, &codec, anchor.sealed_len)?;
    let old_graph = catalogue.graph(&anchor)?;
    let index = catalogue
        .files
        .binary_search_by(|file| file.path.as_slice().cmp(from))
        .map_err(|_| Error::InvalidInput("file not found".into()))?;
    if from != to
        && catalogue
            .files
            .binary_search_by(|file| file.path.as_slice().cmp(to))
            .is_ok()
    {
        return Err(Error::InvalidInput("destination already exists".into()));
    }
    let permissions = permissions.unwrap_or(catalogue.files[index].permissions);
    if from == to && permissions == catalogue.files[index].permissions {
        return Ok(false);
    }
    catalogue.upgrade(&anchor)?;
    catalogue.files[index].path = Zeroizing::new(to.to_vec());
    catalogue.files[index].permissions = permissions;
    catalogue.files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut sealed = anchor.sealed_len;
    let inline =
        anchor.index.primary >= REGION_LEN as u64 && anchor.index.mirror >= REGION_LEN as u64;
    let first = take_slot(
        &mut catalogue.vacant,
        inline.then_some(PRIVATE_START),
        &mut sealed,
    )?;
    let second = take_slot(
        &mut catalogue.vacant,
        inline.then_some(FAILURE_REGION + PRIVATE_START),
        &mut sealed,
    )?;
    for start in [anchor.index.primary, anchor.index.mirror] {
        catalogue.vacant.push(Vacant {
            span: Span {
                start,
                len: anchor.index.len,
            },
            kind: VacantKind::Pending,
        });
    }
    catalogue.vacant.sort_by_key(|entry| entry.span.start);
    let body = catalogue.encode(&codec, sealed)?;
    let encoded = shared::encode_private(archive, mode, key, &body)?;
    let root = RootRef {
        primary: first,
        mirror: second,
        len: encoded.len() as u64,
        digest: strong_checksum(&encoded),
    };
    let commit = shared::commitment(&anchor)?;
    let next = Anchor {
        generation: anchor
            .generation
            .checked_add(1)
            .ok_or(Error::CorruptRecord)?,
        previous: commit,
        sealed_len: sealed,
        index: root,
        object_root: root.digest,
        ..anchor.clone()
    };
    let prepared = shared::prepare(&next, authority, signer)?;
    let next_graph = catalogue.graph(&next)?;
    let plan = old_graph.transition_to(&next_graph)?;
    let old_catalogue = Catalogue::decode(
        &shared::snapshot(storage, archive, mode, authority, key)?.1,
        &codec,
        anchor.sealed_len,
    )?;
    let mut old_catalogue = old_catalogue;
    old_catalogue.upgrade(&anchor)?;
    let mut reservations = Vec::new();
    for span in &plan.writes {
        if span.start >= anchor.sealed_len {
            continue;
        }
        let old = old_catalogue
            .vacant
            .iter()
            .find(|entry| {
                entry.span.start <= span.start
                    && entry.span.start + entry.span.len >= span.start + span.len
            })
            .ok_or(Error::CorruptRecord)?;
        reservations.push(Reservation {
            namespace: match old.kind {
                VacantKind::Free => FREE,
                VacantKind::Pending => PENDING,
            },
            base: old.span.start,
            start: span.start,
            len: span.len,
        });
    }
    old_graph.validate_reservations(&reservations)?;
    let mut journal = InlineSession::open(storage, archive, mode, key)?;
    journal.begin(storage, commit, reservations)?;
    for span in &plan.erase_before_write {
        zero(storage, *span)?;
    }
    if let Some(span) = plan.append {
        if storage.len()? != span.start {
            return Err(Error::CorruptRecord);
        }
        let zeros = [0u8; 65536];
        let mut left = span.len;
        while left > 0 {
            let n = left.min(zeros.len() as u64) as usize;
            storage.append(&zeros[..n])?;
            left -= n as u64;
        }
    }
    storage.write_at(root.primary, &encoded)?;
    storage.write_at(root.mirror, &encoded)?;
    shared::publish(storage, &prepared, authority, commit)?;
    // Re-select the actual persisted graph; never trust a caller's retirement list.
    let recovered = recover(storage, archive, mode, authority, key)?;
    if recovered != next {
        return Err(Error::CorruptRecord);
    }
    Image::open(
        allocation::compaction::View(storage),
        archive,
        mode,
        authority,
        key,
    )?
    .verify_all()?;
    Ok(true)
}
