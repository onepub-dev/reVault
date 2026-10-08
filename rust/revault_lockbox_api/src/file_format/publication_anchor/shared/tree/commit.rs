//! Roll forward a selected published tree. Selected pending claims, not journal
//! reservations or a surviving older publication, authorize retirement. The
//! arena remains allocated (erased pending space) until a future transition.
use super::*;
use crate::file_format::preparation_journal::compact::session::OverflowSession;
use crate::file_format::preparation_journal::{FREE, PENDING};

/// Requires exclusive access throughout. This bounded adapter handles published
/// overflow preparation and its empty-active/idle checkpoints; it does not write
/// a new graph, truncate or recover inline transactions.
pub(crate) fn recover_commit(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    recover_inner(storage, archive, mode, authority, key, false)
}

pub(super) fn recover_writer_commit(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    recover_inner(storage, archive, mode, authority, key, true)
}

fn recover_inner(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    writer_inline: bool,
) -> Result<()> {
    let (anchor, body) = snapshot(storage, archive, mode, authority, key)?;
    let tree = Tree::from_snapshot(storage, archive, mode, key, anchor, &body)?;
    let base = commitment(&tree.anchor)?;
    let mut session = OverflowSession::open(storage, archive, mode, key)?;
    if storage.len()? != tree.anchor.sealed_len {
        return Err(Error::CorruptRecord);
    }
    let arena = session.arena();
    let arena_reservations: Vec<_> = session
        .reservations()
        .iter()
        .copied()
        .filter(|range| {
            arena.is_some_and(|reference| {
                [reference.primary, reference.mirror].contains(&range.start)
                    && range.len == reference.len
            })
        })
        .collect();
    let pending = tree.graph.pending();
    if session.active() {
        if tree.anchor.generation < 2 || session.base() != tree.anchor.previous {
            return Err(Error::CorruptRecord);
        }
        if let Some(reference) = arena {
            // Both copies need explicit, exact selected pending ownership. A
            // digest-valid journal reference alone is never erase authority.
            for start in [reference.primary, reference.mirror] {
                if !pending.contains(&Span {
                    start,
                    len: reference.len,
                }) {
                    return Err(Error::CorruptRecord);
                }
            }
        }
        if arena.is_some() || writer_inline {
            // Reservations have no cleanup authority after publication: they may
            // now hold selected live nodes/payload. Check their bounded syntax
            // and separation, but never erase them as an old free-space map.
            let mut ranges = Vec::new();
            for reservation in session.reservations() {
                let end = reservation
                    .start
                    .checked_add(reservation.len)
                    .ok_or(Error::CorruptRecord)?;
                if !matches!(reservation.namespace, FREE | PENDING)
                    || reservation.len == 0
                    || reservation.base > reservation.start
                    || reservation.start < REGION_LEN as u64
                    || end > tree.anchor.sealed_len
                    || arena.is_some_and(|reference| {
                        [reference.primary, reference.mirror]
                            .into_iter()
                            .any(|start| {
                                reservation.start < start + reference.len
                                    && end > start
                                    && !(writer_inline
                                        && reservation.start == start
                                        && reservation.len == reference.len)
                            })
                    })
                {
                    return Err(Error::CorruptRecord);
                }
                ranges.push((reservation.start, end));
            }
            ranges.sort_unstable();
            if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
                return Err(Error::CorruptRecord);
            }
            if !arena_reservations.is_empty() && arena_reservations.len() != 2 {
                return Err(Error::CorruptRecord);
            }
        } else if !session.reservations().is_empty() {
            return Err(Error::CorruptRecord);
        }
        tree.graph.verify_free(storage)?;
    } else {
        if session.base() != base || arena.is_some() || !session.reservations().is_empty() {
            return Err(Error::CorruptRecord);
        }
        tree.graph.verify_reclaimed(storage)?;
    }
    if !tree.anchor.keys.absent() {
        super::super::super::bootstrap::read_directory_in(storage, &tree.anchor, Layout::Shared)?;
    }
    // All malformed-input checks precede mutation. Repair selected fixed-role
    // dependencies and mirror its publication before erasing older membership.
    tree.mirror_pages(storage)?;
    ensure_mirrored(storage, archive, mode, authority, base)?;
    session.mirror(storage)?;
    if !session.active() {
        return Ok(());
    }
    for span in &pending {
        if arena
            .is_some_and(|reference| [reference.primary, reference.mirror].contains(&span.start))
        {
            continue;
        }
        erase(storage, *span)?;
    }
    storage.sync()?;
    if let Some(reference) = arena {
        if arena_reservations.is_empty() {
            session.unlink_after_cleanup(storage)?;
        } else {
            session.unlink_reused_after_cleanup(storage, arena_reservations)?;
        }
        for start in [reference.primary, reference.mirror] {
            erase(
                storage,
                Span {
                    start,
                    len: reference.len,
                },
            )?;
        }
        storage.sync()?;
    }
    tree.graph.verify_reclaimed(storage)?;
    session.finish(storage, base)
}

fn erase(storage: &mut impl Storage, span: Span) -> Result<()> {
    let end = span
        .start
        .checked_add(span.len)
        .ok_or(Error::CorruptRecord)?;
    let mut position = span.start;
    let zeros = [0; 65536];
    while position < end {
        let n = (end - position).min(zeros.len() as u64) as usize;
        storage.write_at(position, &zeros[..n])?;
        position += n as u64;
    }
    Ok(())
}
