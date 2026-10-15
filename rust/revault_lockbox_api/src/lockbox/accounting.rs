//! Physical ownership follows authenticated references, never scanned payloads.
use super::{Lockbox, LockboxInspector};
use crate::free_slot::FreeSlot;
use crate::page::PageObjectKind;
use crate::storage::Storage;
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct Inventory {
    pub(crate) ranges: BTreeMap<u64, u64>,
    pub(crate) history_bytes: u64,
}
impl Inventory {
    fn add(&mut self, offset: u64, len: u64) -> Result<()> {
        if offset == 0 {
            return Ok(());
        }
        if len == 0 || offset.checked_add(len).is_none() {
            return Err(Error::CorruptRecord);
        }
        if let Some(old) = self.ranges.insert(offset, len) {
            if old != len {
                return Err(Error::CorruptRecord);
            }
        }
        if self.ranges.len() > 1_000_000 {
            return Err(Error::SecurityLimitExceeded(
                "too many allocation references".into(),
            ));
        }
        Ok(())
    }
    fn gaps(&self, free: &[FreeSlot], end: u64) -> Result<Vec<FreeSlot>> {
        let mut ranges: Vec<_> = self
            .ranges
            .iter()
            .map(|(&offset, &len)| FreeSlot { offset, len })
            .chain(free.iter().copied())
            .collect();
        ranges.sort_by_key(|slot| slot.offset);
        let mut cursor = crate::constants::HEADER_LEN as u64;
        let mut gaps = Vec::new();
        for slot in ranges {
            let next = slot
                .offset
                .checked_add(slot.len)
                .ok_or(Error::CorruptRecord)?;
            if slot.offset < cursor || next > end {
                return Err(Error::CorruptRecord);
            }
            if slot.offset > cursor {
                gaps.push(FreeSlot {
                    offset: cursor,
                    len: slot.offset - cursor,
                });
            }
            cursor = next;
        }
        if cursor < end {
            gaps.push(FreeSlot {
                offset: cursor,
                len: end - cursor,
            });
        }
        Ok(gaps)
    }
}
impl<State> Lockbox<State> {
    pub(crate) fn storage_inventory(&self, include_current_control: bool) -> Result<Inventory> {
        let mut inventory = Inventory::default();
        for entry in self.toc_entries.values().filter(|entry| !entry.deleted) {
            if entry.chunks.is_empty() {
                inventory.add(entry.record_offset, entry.record_len)?;
            } else {
                for segment in entry.chunks.iter().flat_map(|chunk| &chunk.segments) {
                    inventory.add(segment.page_offset, segment.page_len)?;
                }
            }
        }
        let mut roots = Vec::new();
        let mut toc_roots = vec![(self.toc_tree.root_offset, self.toc_root_commitment, 0usize)];
        let mut toc_seen = BTreeSet::new();
        while let Some((offset, commitment, depth)) = toc_roots.pop() {
            if offset == 0 {
                continue;
            }
            if depth > 8 || !toc_seen.insert(offset) {
                return Err(Error::CorruptRecord);
            }
            let children = self.with_authenticated_page(offset, commitment, |page| {
                let [object] = page.objects.as_slice() else {
                    return Err(Error::CorruptRecord);
                };
                if !matches!(
                    object.kind,
                    PageObjectKind::TocLeaf | PageObjectKind::TocInternal
                ) {
                    return Err(Error::CorruptRecord);
                }
                object.with_payload(|payload| {
                    match crate::file_format::decode_toc_node(payload)? {
                        crate::file_format::TocNode::Internal(children) => Ok(children),
                        crate::file_format::TocNode::Leaf(_) => Ok(Vec::new()),
                    }
                })?
            })?;
            inventory.add(offset, commitment.physical_len)?;
            toc_roots.extend(
                children
                    .into_iter()
                    .map(|child| (child.offset, child.commitment, depth + 1)),
            );
        }
        let mut variable_roots = vec![
            (
                self.variable_root_offset,
                self.variable_root_commitment,
                0usize,
                true,
            ),
            (
                self.forms.tree.root_offset,
                self.form_root_commitment,
                0usize,
                false,
            ),
        ];
        let mut variable_seen = BTreeSet::new();
        while let Some((offset, commitment, depth, variable)) = variable_roots.pop() {
            if offset == 0 {
                continue;
            }
            if depth > 8 || !variable_seen.insert(offset) {
                return Err(Error::CorruptRecord);
            }
            let children = self.with_authenticated_secure_page(offset, commitment, |page| {
                let [object] = page.objects.as_slice() else {
                    return Err(Error::CorruptRecord);
                };
                if (variable
                    && !matches!(
                        object.kind,
                        PageObjectKind::VariableLeaf | PageObjectKind::VariableInternal
                    ))
                    || (!variable
                        && !matches!(
                            object.kind,
                            PageObjectKind::FormLeaf | PageObjectKind::FormInternal
                        ))
                {
                    return Err(Error::CorruptRecord);
                }
                let payload = object.secure_payload().ok_or(Error::CorruptRecord)?;
                if variable {
                    match crate::variable_btree::decode_variable_node_secure(payload)? {
                        crate::variable_btree::VariableNode::Internal(children) => Ok(children
                            .into_iter()
                            .map(|child| (child.offset, child.commitment))
                            .collect::<Vec<_>>()),
                        crate::variable_btree::VariableNode::Leaf(_) => Ok(Vec::new()),
                    }
                } else {
                    match crate::form_btree::decode_form_node_secure(payload)? {
                        crate::form_btree::FormNode::Internal(children) => Ok(children
                            .into_iter()
                            .map(|child| (child.offset, child.commitment))
                            .collect::<Vec<_>>()),
                        crate::form_btree::FormNode::Leaf(_) => Ok(Vec::new()),
                    }
                }
            })?;
            inventory.add(offset, commitment.physical_len)?;
            variable_roots.extend(
                children
                    .into_iter()
                    .map(|(offset, commitment)| (offset, commitment, depth + 1, variable)),
            );
        }
        if include_current_control && self.commit_root_offset != 0 {
            let (auth, digest) = self.read_and_verify_commit_auth_at(self.commit_auth_offset)?;
            if digest != self.commit_auth_digest
                || auth.commit_root_offset != self.commit_root_offset
            {
                return Err(Error::CorruptRecord);
            }
            let root = self.read_verified_commit_root_from_auth(&auth)?;
            roots.push((
                root.free_index_root_offset,
                root.free_index_root_commitment,
                0usize,
            ));
            roots.push((
                root.post_cleanup_free_index_root_offset,
                root.post_cleanup_free_index_root_commitment,
                0usize,
            ));
        }
        let mut visited = BTreeSet::new();
        while let Some((offset, commitment, depth)) = roots.pop() {
            if offset == 0 {
                continue;
            }
            if depth > 8 {
                return Err(Error::CorruptRecord);
            }
            if !visited.insert(offset) {
                continue;
            }
            let children = self.with_authenticated_page(offset, commitment, |page| {
                let [object] = page.objects.as_slice() else {
                    return Err(Error::CorruptRecord);
                };
                match object.kind {
                    PageObjectKind::FreeIndexInternal => {
                        object.with_payload(crate::free_index::decode_free_index_internal)?
                    }
                    PageObjectKind::FreeIndexLeaf => Ok(Vec::new()),
                    _ => Err(Error::CorruptRecord),
                }
            })?;
            inventory.add(offset, commitment.physical_len)?;
            roots.extend(
                children
                    .into_iter()
                    .map(|child| (child.offset, child.commitment, depth + 1)),
            );
        }
        for offset in self.key_directory.offsets() {
            if offset != 0 {
                inventory.add(offset, self.key_directory.physical_len)?;
            }
        }
        let mut auth_offset = self.commit_auth_offset;
        let mut expected_auth_digest = self.commit_auth_digest;
        let mut seen_auth = BTreeSet::new();
        while auth_offset != 0 {
            if !seen_auth.insert(auth_offset) || seen_auth.len() > 1_000_000 {
                return Err(Error::CorruptRecord);
            }
            // Chain digests retain the already established owner authority;
            // exact control buffers establish canonical allocation extents.
            let (payload, auth_len) =
                self.read_control_payload_at(auth_offset, PageObjectKind::CommitAuth)?;
            if crate::commit_auth::commit_auth_digest(&payload) != expected_auth_digest {
                return Err(Error::CorruptRecord);
            }
            let auth = crate::commit_auth::decode_commit_auth(&payload)?;
            let (root_payload, root_len) =
                self.read_control_payload_at(auth.commit_root_offset, PageObjectKind::CommitRoot)?;
            if crate::crypto::strong_checksum(&root_payload) != auth.commit_root_digest {
                return Err(Error::CorruptRecord);
            }
            if auth.previous_auth_offset >= auth_offset {
                return Err(Error::CorruptRecord);
            }
            expected_auth_digest = auth.previous_auth_digest;
            inventory.add(auth_offset, auth_len)?;
            inventory.add(auth.commit_root_offset, root_len)?;
            inventory.history_bytes += auth_len + root_len;
            auth_offset = auth.previous_auth_offset;
        }
        if include_current_control {
            let mut offset = self.redaction_manifest_offset;
            let mut commitment = self.redaction_manifest_commitment;
            let mut seen = BTreeSet::new();
            while offset != 0 {
                if !seen.insert(offset)
                    || seen.len() > crate::file_format::redaction_manifest::MAX_REDACTION_PAGES
                {
                    return Err(Error::CorruptRecord);
                }
                let page = self.read_redaction_manifest_page_at(offset, commitment)?;
                inventory.add(offset, commitment.physical_len)?;
                offset = page.next_page_offset;
                commitment = page.next_page_commitment;
            }
        }
        Ok(inventory)
    }

    pub(crate) fn retire_unreferenced_allocations(&mut self) -> Result<()> {
        let inventory = self.storage_inventory(false)?;
        // Retirement happens only after the new publication. The base free
        // index and obsolete control pages remain intact throughout preparation.
        self.redacted_free_slots =
            inventory.gaps(&self.free_space.slots_by_offset(), self.storage.len()?)?;
        Ok(())
    }

    pub(crate) fn validate_base_reservations(&self) -> Result<()> {
        if self.free_space.slots_by_offset().is_empty() {
            return Ok(());
        }
        self.storage_inventory(true)?.gaps(
            &self.free_space.slots_by_offset(),
            self.transaction_start_len,
        )?;
        Ok(())
    }

    pub(crate) fn validate_cleanup_free_slots(&self, slots: &[FreeSlot]) -> Result<()> {
        self.storage_inventory(true)?
            .gaps(slots, self.storage.len()?)?;
        Ok(())
    }
}
impl<State> LockboxInspector<'_, State> {
    /// Verify complete physical ownership and zero-filled reusable ranges.
    /// This is read-only and never promotes orphan contents into the archive.
    /// Commit or abort staged preparation first; pending recovery, invalid
    /// extents, unaccounted storage and nonzero reusable ranges are errors.
    ///
    /// ```no_run
    /// # fn inspect(archive: &revault_lockbox_api::Lockbox<revault_lockbox_api::ReadOnly>) -> revault_lockbox_api::Result<()> {
    /// // Inspect a separately opened, explicitly read-only archive.
    /// archive.inspector().verify_storage()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_storage(&self) -> Result<()> {
        let lb = self.lockbox;
        lb.require_clean_transaction()?;
        if lb.preparing {
            return Err(Error::InvalidOperation(
                "commit or abort before checking sealed storage".into(),
            ));
        }
        let inventory = lb.storage_inventory(true)?;
        if !inventory
            .gaps(&lb.free_space.slots_by_offset(), lb.storage.len()?)?
            .is_empty()
        {
            return Err(Error::InvalidOperation("unaccounted storage ranges".into()));
        }
        for slot in lb.free_space.slots_by_offset() {
            let mut offset = slot.offset;
            let end = offset.checked_add(slot.len).ok_or(Error::CorruptRecord)?;
            while offset < end {
                let len = (end - offset).min(64 * 1024) as usize;
                if lb
                    .storage
                    .read_at(offset, len)?
                    .iter()
                    .any(|byte| *byte != 0)
                {
                    return Err(Error::InvalidOperation("nonzero reusable storage".into()));
                }
                offset += len as u64;
            }
        }
        Ok(())
    }
}

impl Lockbox<crate::Writable> {
    /// Reserve control extents before serializing either free-index snapshot.
    /// Otherwise an index could accidentally advertise its own page as free.
    pub(crate) fn reserve_control_pages(&mut self) -> Result<()> {
        fn tree_pages(slots: usize) -> usize {
            let mut level = slots.div_ceil(crate::free_index::FREE_INDEX_LEAF_SLOT_CAPACITY);
            let mut pages = level;
            while level > 1 {
                level = level.div_ceil(crate::free_index::FREE_INDEX_INTERNAL_CHILD_CAPACITY);
                pages += level;
            }
            pages
        }
        let ranges =
            crate::file_format::redaction_manifest::bounded_ranges(&self.redacted_free_slots)?;
        let manifests = ranges
            .len()
            .div_ceil(crate::file_format::redaction_manifest::RANGES_PER_PAGE);
        let needed = |free: &crate::free_slot::FreeSpace| {
            let mut post = free.clone();
            for slot in &ranges {
                post.add(*slot);
            }
            tree_pages(free.slots_by_offset().len())
                + if ranges.is_empty() {
                    0
                } else {
                    tree_pages(post.slots_by_offset().len())
                }
                + manifests
        };
        let initial = self.free_space.clone();
        if self.format_mode.unpadded() {
            // Variable-size control pages cannot reserve a fixed padded slot.
            // Append this batch so neither free-index snapshot advertises its
            // own storage. All retired extents remain available for data reuse.
            self.control_reservations = std::iter::repeat_n(None, needed(&initial)).collect();
            return Ok(());
        }
        let mut count = needed(&initial);
        // Taking an exact-size free slot may reduce the number of index nodes.
        // Solve that dependency before making any reservation visible in RAM.
        for _ in 0..8 {
            let mut free = initial.clone();
            let offsets: std::collections::VecDeque<_> = (0..count)
                .map(|_| {
                    free.allocate(crate::constants::DEFAULT_METADATA_PAGE_BYTES as u64)
                        .map(|slot| slot.offset)
                })
                .collect();
            let actual = needed(&free);
            if actual == count {
                self.free_space = free;
                self.control_reservations = offsets;
                return Ok(());
            }
            count = actual;
        }
        // At a grouping boundary the count can oscillate. Append this bounded
        // batch; a later transaction can reuse those complete extents.
        self.control_reservations = std::iter::repeat_n(None, needed(&initial)).collect();
        Ok(())
    }

    pub(crate) fn write_control_page(
        &mut self,
        kind: PageObjectKind,
        payload: Vec<u8>,
    ) -> Result<(u64, crate::file_format::page_commitment::PageCommitment)> {
        let reservation = self
            .control_reservations
            .pop_front()
            .ok_or(Error::CorruptRecord)?;
        let offset = match reservation {
            Some(offset) => offset,
            None => self.next_append_page_offset()?,
        };
        let objects = vec![crate::page::PageObject::new(kind, self.sequence, payload)];
        let page_size = if self.format_mode.unpadded() {
            crate::page::page_size_for_encoded_objects_with_format(&objects, self.format_mode)?
        } else {
            crate::constants::DEFAULT_METADATA_PAGE_BYTES
        };
        let page = crate::page::DecodedPage {
            page_id: offset,
            sequence: self.sequence,
            objects,
        };
        let commitment = crate::file_format::page_commitment::PageCommitment::from_page(
            self.lockbox_id,
            self.format_mode,
            page_size as u64,
            &page,
        )?;
        self.page_manager
            .borrow_mut()
            .stage_decoded_page_with_policy(
                offset,
                page_size,
                page,
                crate::page_cache::PageWritePolicy::RetainAfterFlush,
            )?;
        Ok((offset, commitment))
    }
}
