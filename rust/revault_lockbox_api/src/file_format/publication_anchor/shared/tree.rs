//! Authenticated overflow-index reader. This connects selected private authority
//! to bounded tree traversal and physical ownership, not public record semantics.
//! The caller retains a stable snapshot/read lock for open and subsequent reads.
use super::*;
use crate::file_format::allocation_map::Extent;
use crate::file_format::authenticated_index::{Entry, Index, Visit};
use ownership::{Graph, Span, Vacant, VacantKind};

const MAGIC: &[u8; 8] = b"RV4TRE01";
const OWNERSHIP: u8 = 0;
const MAX_PAGES: usize = 4096;
pub(crate) const MAX_OWNERSHIP_RECORDS: usize = 4096;

pub(crate) struct Tree {
    pub anchor: Anchor,
    pub graph: Graph,
    index: Index,
    root: RootRef,
    pages: Vec<RootRef>,
}

/// Fixed-size private manifest. The selected publication authenticates these
/// bytes through the existing private envelope, including the complete tree root.
pub(crate) fn manifest(root: RootRef) -> [u8; 64] {
    let mut body = [0; 64];
    body[..8].copy_from_slice(MAGIC);
    body[8..16].copy_from_slice(&root.primary.to_le_bytes());
    body[16..24].copy_from_slice(&root.mirror.to_le_bytes());
    body[24..32].copy_from_slice(&root.len.to_le_bytes());
    body[32..].copy_from_slice(&root.digest);
    body
}

fn parse_manifest(body: &[u8]) -> Result<RootRef> {
    if body.len() != 64 || &body[..8] != MAGIC {
        return Err(Error::CorruptRecord);
    }
    Ok(RootRef {
        primary: u64::from_le_bytes(body[8..16].try_into().unwrap()),
        mirror: u64::from_le_bytes(body[16..24].try_into().unwrap()),
        len: u64::from_le_bytes(body[24..32].try_into().unwrap()),
        digest: body[32..].try_into().unwrap(),
    })
}

impl Tree {
    /// Read-only salvage authenticates selected membership independently of
    /// payload availability and unfinished cleanup. Never search old roots.
    /// Stage typed records during the authenticated ownership walk. The caller
    /// must discard staged output unless this entire operation succeeds.
    pub(crate) fn salvage_visit(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        visitor: impl FnMut(Entry) -> Result<()>,
    ) -> Result<Self> {
        let (anchor, body) = salvage_private(storage, archive, mode, authority, key)?;
        Self::from_snapshot_visit(
            storage,
            archive,
            mode,
            key,
            anchor,
            &body,
            visitor,
            &mut |_| {},
        )
    }
    pub(crate) fn open(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
    ) -> Result<Self> {
        Self::open_visit(storage, archive, mode, authority, key, |_| Ok(()))
    }

    /// Same complete ownership and reclaimed-space validation as `open`; typed
    /// decoding shares the traversal without retaining an extra record cache.
    pub(crate) fn open_visit(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        visitor: impl FnMut(Entry) -> Result<()>,
    ) -> Result<Self> {
        Self::open_visit_observed(storage, archive, mode, authority, key, visitor, |_| {})
    }

    /// Diagnostic stages preserve the ordinary complete validation path.
    pub(crate) fn open_visit_observed(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        visitor: impl FnMut(Entry) -> Result<()>,
        mut stage: impl FnMut(&'static str),
    ) -> Result<Self> {
        let (anchor, body) = open_private(storage, archive, mode, authority, key)?;
        stage("selected_publication");
        let tree = Self::from_snapshot_visit(
            storage, archive, mode, key, anchor, &body, visitor, &mut stage,
        )?;
        tree.graph.verify_reclaimed(storage)?;
        stage("reclaimed_space");
        Ok(tree)
    }

    fn from_snapshot(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
        anchor: Anchor,
        body: &[u8],
    ) -> Result<Self> {
        Self::from_snapshot_visit(
            storage,
            archive,
            mode,
            key,
            anchor,
            body,
            |_| Ok(()),
            &mut |_| {},
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_snapshot_visit(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
        anchor: Anchor,
        body: &[u8],
        mut visitor: impl FnMut(Entry) -> Result<()>,
        stage: &mut impl FnMut(&'static str),
    ) -> Result<Self> {
        let root = parse_manifest(body)?;
        let index = Index::new(archive, mode, key)?;
        let mut pages = Vec::new();
        let mut packs = Vec::new();
        let mut vacant = Vec::new();
        if anchor.keys.absent() {
            for bank in [0, FAILURE_REGION] {
                vacant.push(Vacant {
                    span: Span {
                        start: bank + KEYS_START,
                        len: 4096,
                    },
                    kind: VacantKind::Free,
                });
            }
        }
        let mut ownership_records = 0;
        index.visit_owned(storage, root, anchor.sealed_len, |event| {
            match event {
                Visit::Page(reference) => {
                    if pages.len() == MAX_PAGES {
                        return Err(Error::SecurityLimitExceeded(
                            "overflow page ownership bound".into(),
                        ));
                    }
                    // Each node copy owns one aligned failure-region allocation.
                    // Short unpadded node tails are derived zero/free space, not
                    // another caller-controlled ownership record or hidden data.
                    if reference.len == 0 || reference.len > FAILURE_REGION {
                        return Err(Error::CorruptRecord);
                    }
                    for start in [reference.primary, reference.mirror] {
                        if start % FAILURE_REGION != 0 {
                            return Err(Error::CorruptRecord);
                        }
                        if reference.len < FAILURE_REGION {
                            vacant.push(Vacant {
                                span: Span {
                                    start: start
                                        .checked_add(reference.len)
                                        .ok_or(Error::CorruptRecord)?,
                                    len: FAILURE_REGION - reference.len,
                                },
                                kind: VacantKind::Free,
                            });
                        }
                    }
                    pages.push(reference);
                }
                Visit::Entry(entry) if entry.namespace == OWNERSHIP => {
                    ownership_records += 1;
                    if ownership_records > MAX_OWNERSHIP_RECORDS {
                        return Err(Error::SecurityLimitExceeded(
                            "overflow allocation record bound".into(),
                        ));
                    }
                    if entry.key.len() != 9 || entry.value.len() < 8 {
                        return Err(Error::CorruptRecord);
                    }
                    let span = Span {
                        start: u64::from_be_bytes(entry.key[1..].try_into().unwrap()),
                        len: u64::from_le_bytes(entry.value[..8].try_into().unwrap()),
                    };
                    match (entry.key[0], entry.value.len()) {
                        (0 | 1, 8) => vacant.push(Vacant {
                            span,
                            kind: if entry.key[0] == 0 {
                                VacantKind::Free
                            } else {
                                VacantKind::Pending
                            },
                        }),
                        (2, 40) => packs.push(Extent {
                            start: span.start,
                            len: span.len,
                            digest: entry.value[8..].try_into().unwrap(),
                        }),
                        _ => return Err(Error::CorruptRecord),
                    }
                }
                Visit::Entry(entry) => visitor(entry)?,
            }
            Ok(())
        })?;
        stage("index_walk_and_decode");
        let graph = Graph::derive_with_descendants(&anchor, &packs, &vacant, &pages)?;
        stage("ownership_graph");
        Ok(Self {
            anchor,
            graph,
            index,
            root,
            pages,
        })
    }

    /// Only call after complete graph and operation-specific validation. Each
    /// address is owned by the selected authenticated descendant traversal.
    fn mirror_pages(&self, storage: &mut impl Storage) -> Result<()> {
        for reference in &self.pages {
            let bytes = crate::page_buffer::ZeroizingBytes::new(reference.read_verified(storage)?);
            for start in [reference.primary, reference.mirror] {
                if strong_checksum(&storage.read_at(start, reference.len as usize)?)
                    != reference.digest
                {
                    storage.write_at(start, &bytes)?;
                }
            }
        }
        storage.sync()
    }

    /// Returns authenticated raw records. A typed adapter must still validate
    /// each record's public semantics and any referenced payload contents.
    pub(crate) fn get(
        &self,
        storage: &impl Storage,
        namespace: u8,
        key: &[u8],
    ) -> Result<Option<Entry>> {
        if namespace == OWNERSHIP {
            return Err(Error::InvalidInput("reserved ownership namespace".into()));
        }
        self.index
            .get(storage, self.root, self.anchor.sealed_len, namespace, key)
    }

    /// Callers stage output until success; a late I/O or sink failure invalidates
    /// the whole batch. The storage must remain the same locked snapshot.
    pub(crate) fn visit(
        &self,
        storage: &impl Storage,
        mut visitor: impl FnMut(Entry) -> Result<()>,
    ) -> Result<()> {
        self.index
            .visit(storage, self.root, self.anchor.sealed_len, |entry| {
                if entry.namespace != OWNERSHIP {
                    visitor(entry)?;
                }
                Ok(())
            })
    }
}

#[cfg(test)]
mod tests;

mod abort;
pub(crate) use abort::recover_abort;

mod commit;
pub(crate) use commit::recover_commit;

mod update;
pub(crate) use update::{
    grow_dense_payload_records, grow_dense_records, recover_update, relocate_records,
    rewrite_payload_records, rewrite_prepared_secure_payload_records,
    rewrite_prepared_storage_payload_records, rewrite_records, rewrite_secure_payload_records,
    PayloadPlan, PreparedSecurePayloadPlan, PreparedStoragePayloadPlan, ReadView,
    SecurePayloadPlan, SelectedSource, StoragePayloadCallback,
};
