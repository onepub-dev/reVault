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
        let mut roots = vec![
            (self.toc_tree.root_offset, 0u8, 0usize),
            (self.variable_root_offset, 1, 0),
            (self.forms.tree.root_offset, 2, 0),
        ];
        if include_current_control && self.commit_root_offset != 0 {
            let root = self.read_commit_root_at(self.commit_root_offset)?;
            roots.push((root.free_index_root_offset, 3, 0));
            roots.push((root.post_cleanup_free_index_root_offset, 3, 0));
        }
        let mut visited = BTreeSet::new();
        while let Some((offset, kind, depth)) = roots.pop() {
            if offset == 0 {
                continue;
            }
            if depth > 8 {
                return Err(Error::CorruptRecord);
            }
            if !visited.insert(offset) {
                continue;
            }
            let children: Vec<u64> = if kind == 1 || kind == 2 {
                self.with_secure_page(offset, |page| {
                    if page.objects.len() != 1 {
                        return Err(Error::CorruptRecord);
                    }
                    let object = &page.objects[0];
                    let payload = object.secure_payload().ok_or(Error::CorruptRecord)?;
                    if kind == 1 {
                        match crate::variable_btree::decode_variable_node_secure(payload)? {
                            crate::variable_btree::VariableNode::Internal(children) => {
                                Ok(children.into_iter().map(|child| child.offset).collect())
                            }
                            crate::variable_btree::VariableNode::Leaf(_) => Ok(Vec::new()),
                        }
                    } else {
                        match crate::form_btree::decode_form_node_secure(payload)? {
                            crate::form_btree::FormNode::Internal(children) => {
                                Ok(children.into_iter().map(|child| child.offset).collect())
                            }
                            crate::form_btree::FormNode::Leaf(_) => Ok(Vec::new()),
                        }
                    }
                })?
            } else {
                let page = self.read_page(offset)?;
                if page.objects.len() != 1 {
                    return Err(Error::CorruptRecord);
                }
                let object = &page.objects[0];
                match (kind, object.kind) {
                    (0, PageObjectKind::TocInternal) => {
                        object.with_payload(
                            |bytes| match crate::file_format::decode_toc_node(bytes)? {
                                crate::file_format::TocNode::Internal(children) => {
                                    Ok(children.into_iter().map(|child| child.offset).collect())
                                }
                                _ => Err(Error::CorruptRecord),
                            },
                        )??
                    }
                    (3, PageObjectKind::FreeIndexInternal) => object
                        .with_payload(crate::free_index::decode_free_index_internal)??
                        .into_iter()
                        .map(|child| child.offset)
                        .collect(),
                    (0, PageObjectKind::TocLeaf) | (3, PageObjectKind::FreeIndexLeaf) => Vec::new(),
                    _ => return Err(Error::CorruptRecord),
                }
            };
            let header = self.storage.read_at(offset, crate::page::PAGE_HEADER_LEN)?;
            inventory.add(
                offset,
                crate::page::physical_page_size_from_page_slice(&header)? as u64,
            )?;
            roots.extend(children.into_iter().map(|child| (child, kind, depth + 1)));
        }
        for offset in self.key_directory.offsets() {
            if offset != 0 {
                inventory.add(offset, self.page_len_at(offset)?)?;
            }
        }
        let mut auth_offset = self.commit_auth_offset;
        let mut seen_auth = BTreeSet::new();
        while auth_offset != 0 {
            if !seen_auth.insert(auth_offset) || seen_auth.len() > 1_000_000 {
                return Err(Error::CorruptRecord);
            }
            // The publication's lineage was verified when opening this handle.
            // Reading ownership must not repeat every hybrid signature check.
            let auth = crate::commit_auth::decode_commit_auth(
                &self.read_commit_auth_payload_at(auth_offset)?,
            )?;
            let auth_len = self.page_len_at(auth_offset)?;
            let root_len = self.page_len_at(auth.commit_root_offset)?;
            inventory.add(auth_offset, auth_len)?;
            inventory.add(auth.commit_root_offset, root_len)?;
            inventory.history_bytes += auth_len + root_len;
            auth_offset = auth.previous_auth_offset;
        }
        if include_current_control {
            let mut offset = self.redaction_manifest_offset;
            let mut seen = BTreeSet::new();
            while offset != 0 {
                if !seen.insert(offset)
                    || seen.len() > crate::file_format::redaction_manifest::MAX_REDACTION_PAGES
                {
                    return Err(Error::CorruptRecord);
                }
                let page = self.read_redaction_manifest_page_at(offset)?;
                inventory.add(offset, self.page_len_at(offset)?)?;
                offset = page.next_page_offset;
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
    ) -> Result<u64> {
        let reservation = self
            .control_reservations
            .pop_front()
            .ok_or(Error::CorruptRecord)?;
        let offset = match reservation {
            Some(offset) => offset,
            None => self.next_append_page_offset()?,
        };
        self.page_manager
            .borrow_mut()
            .stage_decoded_page_with_policy(
                offset,
                crate::constants::DEFAULT_METADATA_PAGE_BYTES,
                crate::page::DecodedPage {
                    page_id: offset,
                    sequence: self.sequence,
                    objects: vec![crate::page::PageObject::new(kind, self.sequence, payload)],
                },
                crate::page_cache::PageWritePolicy::RetainAfterFlush,
            )?;
        Ok(offset)
    }
}
