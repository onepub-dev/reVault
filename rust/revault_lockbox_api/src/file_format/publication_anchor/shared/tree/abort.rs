//! Bounded abort recovery for the appended OverflowSession arena. No committed
//! transition, arena relocation, payload mutation, or public API is enabled here.
use super::*;
use crate::file_format::preparation_journal::compact::session::OverflowSession;

/// Hold exclusive access throughout this call. Authority and all erase ranges
/// are obtained anew from storage, never from a caller-supplied graph or arena.
/// Supports the staging/unlink checkpoints of the two-region tail session only.
pub(crate) fn recover_abort(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
) -> Result<()> {
    recover_inner(storage, archive, mode, authority, key, false)
}

pub(super) fn recover_writer_abort(
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
    writer_tail: bool,
) -> Result<()> {
    let (anchor, body) = snapshot(storage, archive, mode, authority, key)?;
    let tree = if writer_tail && !body.starts_with(MAGIC) {
        let graph = crate::file_format::candidate_files::tree_image::dense_recovery_graph(
            &anchor, &body, key,
        )?;
        Tree {
            anchor,
            graph,
            index: Index::new(archive, mode, key)?,
            root: RootRef::default(),
            pages: Vec::new(),
        }
    } else {
        Tree::from_snapshot(storage, archive, mode, key, anchor, &body)?
    };
    let base = commitment(&tree.anchor)?;
    let mut session = OverflowSession::open(storage, archive, mode, key)?;
    let sealed = tree.anchor.sealed_len;
    let physical = storage.len()?;
    let primary = sealed
        .checked_add(FAILURE_REGION - 1)
        .ok_or(Error::CorruptRecord)?
        / FAILURE_REGION
        * FAILURE_REGION;
    let mirror = primary
        .checked_add(FAILURE_REGION)
        .ok_or(Error::CorruptRecord)?;
    let limit = mirror
        .checked_add(FAILURE_REGION)
        .ok_or(Error::CorruptRecord)?;
    let physical_limit = if writer_tail {
        sealed
            .checked_add(super::update::MAX_APPEND)
            .ok_or(Error::CorruptRecord)?
    } else {
        limit
    };
    if session.base() != base || physical < sealed || physical > physical_limit {
        return Err(Error::CorruptRecord);
    }
    if !session.active()
        && (physical != sealed || session.arena().is_some() || !session.reservations().is_empty())
    {
        return Err(Error::CorruptRecord);
    }
    let mut arena_reservations = Vec::new();
    if let Some(arena) = session.arena() {
        // Canonical appended copies are disjoint from the complete sealed graph
        // (including every reservation), from each other and from control slots.
        // A valid digest alone never authorizes erasing an arbitrary address.
        let reused = writer_tail
            && [arena.primary, arena.mirror].into_iter().all(|start| {
                start
                    .checked_add(arena.len)
                    .is_some_and(|end| end <= sealed)
            });
        if reused {
            for start in [arena.primary, arena.mirror] {
                arena_reservations.push(
                    *session
                        .reservations()
                        .iter()
                        .find(|range| range.start == start && range.len == arena.len)
                        .ok_or(Error::CorruptRecord)?,
                );
            }
        } else if arena.primary != primary
            || arena.mirror != mirror
            || arena.len != FAILURE_REGION
            || physical < limit
            || (!writer_tail && physical != limit)
        {
            return Err(Error::CorruptRecord);
        }
    }
    let reservations = tree
        .graph
        .verify_abort_reservations(storage, session.reservations())?;
    // Finish all read-only validation before the first persistent operation.
    if !tree.anchor.keys.absent() {
        super::super::super::bootstrap::read_directory_in(storage, &tree.anchor, Layout::Shared)?;
    }
    // Retire an older surviving publication before changing any selected vacant
    // bytes. It may still name these bytes as live metadata. The complete graph
    // above also proves repair addresses for selected private/key dependencies.
    ensure_mirrored(storage, archive, mode, authority, base)?;
    session.mirror(storage)?;
    if !session.active() {
        return Ok(());
    }
    let mut cleanup: Vec<Span> = Vec::new();
    for span in reservations {
        if arena_reservations
            .iter()
            .any(|range| range.start == span.start && range.len == span.len)
        {
            continue;
        }
        if let Some(last) = cleanup
            .last_mut()
            .filter(|last| last.start + last.len == span.start)
        {
            last.len += span.len;
        } else {
            cleanup.push(span);
        }
    }
    for span in cleanup {
        erase(storage, span.start, span.start + span.len)?;
    }
    storage.sync()?;
    tree.graph
        .verify_abort_reservations(storage, &arena_reservations)?;
    if session.arena().is_some() {
        if arena_reservations.is_empty() {
            session.unlink_after_cleanup(storage)?;
        } else {
            session.unlink_reused_after_cleanup(storage, arena_reservations.clone())?;
        }
    }
    for range in arena_reservations {
        erase(storage, range.start, range.start + range.len)?;
    }
    storage.sync()?;
    tree.graph.verify_reclaimed(storage)?;
    // With an empty active stub, mirror above also durably removes any older
    // linked stub left by an interrupted unlink before the tail can be erased.
    erase(storage, sealed, physical)?;
    storage.sync()?;
    storage.truncate(sealed)?;
    storage.sync()?;
    session.finish(storage, base)
}

fn erase(storage: &mut impl Storage, mut position: u64, end: u64) -> Result<()> {
    let zeros = [0; 65536];
    while position < end {
        let n = (end - position).min(zeros.len() as u64) as usize;
        storage.write_at(position, &zeros[..n])?;
        position += n as u64;
    }
    Ok(())
}
