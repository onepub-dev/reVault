use super::{Lockbox, StagedLockboxState};
use crate::checked::read_u32_le;
use crate::commit_auth::{commit_auth_digest, commit_auth_message, encode_commit_auth, CommitAuth};
use crate::commit_root::{encode_commit_root, CommitRoot};
use crate::file_format::current_header::publish_header;
use crate::file_format::header_v2::{self, Publication};
use crate::file_format::redaction_manifest::{
    bounded_ranges, encode_page as encode_redaction_manifest_page, RedactionManifestPage,
    RANGES_PER_PAGE,
};
use crate::file_format::{
    encode_toc_internal, encode_toc_leaf, toc_child_groups, toc_leaf_groups, TocChild, TocInternal,
    TocLeaf, TocTreeNode,
};
use crate::free_index::{
    encode_free_index_internal, encode_free_index_leaf, free_index_child_groups,
    free_index_leaf_groups, FreeIndexChild,
};
use crate::incremental_btree::{IncrementalBTree, LeafRewrite};
use crate::key_directory::encode_key_directory;
use crate::page::{page_size_for_encoded_objects, PageObject, PageObjectKind};
use crate::storage::Storage;
use crate::{Error, Result};

const KEY_DIRECTORY_MIRROR_PREFERRED_MIN_DISTANCE: u64 = 1024 * 1024;

impl Lockbox<crate::Writable> {
    /// Persist pending lockbox changes atomically to the backing storage.
    ///
    /// Returns storage, encoding, or security-limit errors if pending changes
    /// cannot be written. Errors before publication roll in-memory metadata
    /// back to the state before the attempt. If publication succeeds but
    /// durable redaction cleanup is interrupted, returns
    /// [`Error::RecoveryRequired`]; the published logical state remains in
    /// force and must be completed with `Lockbox::recover_transaction`.
    pub fn commit(&mut self) -> Result<()> {
        self.require_clean_transaction()?;
        if self.read_only {
            return Err(Error::InvalidOperation(
                "contact-opened lockboxes are read-only; copy the lockbox before editing"
                    .to_string(),
            ));
        }
        if let Some(reason) = &self.poisoned {
            return Err(Error::InvalidOperation(format!(
                "lockbox has an unresolved failed write: {reason}"
            )));
        }
        if self.access_widening_pending {
            // Rebuild into fresh storage before publishing a new decryptor.
            // This excludes abandoned prepare pages and all prior redacted
            // storage from the archive the new recipient can inspect.
            return self.compact();
        }
        let rollback = CommitRollback::capture(self);
        match self.commit_inner() {
            Ok(()) => Ok(()),
            Err(err) => {
                if self.poisoned.is_some() {
                    return Err(err);
                }
                if self.header_generation > rollback.0.header_generation {
                    if let Some(status) = self.transaction_recovery_status() {
                        return Err(Error::RecoveryRequired {
                            transaction_sequence: status.transaction_sequence,
                            range_count: status.range_count,
                            completed_ranges: status.completed_ranges,
                            page_count: status.page_count,
                            completed_pages: status.completed_pages,
                            total_bytes: status.total_bytes,
                            completed_bytes: status.completed_bytes,
                        });
                    }
                    return Err(err);
                }
                rollback.restore(self);
                Err(err)
            }
        }
    }

    fn commit_inner(&mut self) -> Result<()> {
        self.flush_pending_small_files()?;
        self.flush_pending_symlinks()?;
        if self.needs_packing && !self.should_discard_file_pages_after_flush() {
            self.pack_small_file_pages()?;
            self.needs_packing = false;
        } else if self.needs_packing {
            self.needs_packing = false;
        }
        self.stage_variable_tree_redactions()?;
        self.stage_form_tree_redactions()?;
        self.apply_pending_redactions()?;
        if self.toc_tree.root.is_some()
            && self.toc_tree.dirty_keys.is_empty()
            && !self.dirty_variables
            && !self.forms.dirty
            && !self.key_directory.dirty
            && !self.has_dirty_pages()
        {
            return Ok(());
        }
        self.variable_root_offset = self.commit_variable_tree()?;
        self.forms.tree.root_offset = self.commit_form_tree()?;
        self.toc_tree.root_offset = self.commit_toc_btree()?;
        let toc_root_offset = self.toc_tree.root_offset;
        self.free_index_offset = self.write_free_index()?;
        self.post_cleanup_free_index_offset = self.write_post_cleanup_free_index()?;
        self.sequence += 1;
        let redaction_ranges = self.redacted_free_slots.clone();
        let (manifest_offset, range_count, total_bytes) =
            self.append_redaction_manifest_pages(&redaction_ranges)?;
        self.redaction_manifest_offset = manifest_offset;
        self.redaction_range_count = range_count;
        self.redaction_total_bytes = total_bytes;
        self.cleanup_completed_ranges = 0;
        self.cleanup_completed_pages = 0;
        self.cleanup_completed_bytes = 0;
        self.flush_dirty_pages()?;
        self.write_key_directory_mirrors_if_dirty()?;
        let previous_commit_auth_offset = self.commit_auth_offset;
        let previous_commit_auth_digest = self.commit_auth_digest;
        let commit_root_payload = encode_commit_root(&CommitRoot {
            sequence: self.sequence,
            toc_root_offset,
            variable_root_offset: self.variable_root_offset,
            form_root_offset: self.forms.tree.root_offset,
            free_index_root_offset: self.free_index_offset,
            post_cleanup_free_index_root_offset: self.post_cleanup_free_index_offset,
            key_directory_offset: self.key_directory.primary_offset,
            key_directory_mirror_offset: self.key_directory.mirror_offset,
            key_directory_generation: self.key_directory.generation,
            previous_commit_root_offset: self.commit_root_offset,
            redaction_manifest_offset: self.redaction_manifest_offset,
            redaction_range_count: u64::from(self.redaction_range_count),
            redaction_total_bytes: self.redaction_total_bytes,
            flags: 0,
        });
        let commit_root_digest = crate::crypto::strong_checksum(&commit_root_payload);
        self.commit_root_offset = self.append_commit_root_page(commit_root_payload)?;
        let lockbox_id = self.lockbox_id;
        let sequence = self.sequence;
        let commit_root_offset = self.commit_root_offset;
        let signer = self.require_owner_signing_key()?;
        let mut auth = CommitAuth {
            lockbox_id,
            sequence,
            commit_root_offset,
            commit_root_digest,
            previous_auth_offset: previous_commit_auth_offset,
            previous_auth_digest: previous_commit_auth_digest,
            flags: 0,
            signatures: signer.empty_signatures(),
        };
        let message = commit_auth_message(&auth)?;
        auth.signatures = signer.sign(&message);
        let commit_auth_payload = encode_commit_auth(&auth)?;
        self.commit_auth_digest = commit_auth_digest(&commit_auth_payload);
        self.commit_auth_offset = self.append_commit_auth_page(commit_auth_payload)?;
        self.flush_dirty_pages()?;
        self.storage.sync()?;
        let pending_cleanup = self.redaction_manifest_offset != 0;
        let cleanup_sequence = if pending_cleanup {
            self.cleanup_sequence
        } else {
            self.sequence
        };
        self.publish_transaction_header(cleanup_sequence)?;
        if pending_cleanup {
            self.cleanup_published_redactions(|_| {})?;
            self.publish_transaction_header(self.sequence)?;
            self.free_index_offset = self.post_cleanup_free_index_offset;
        }
        self.publish_redacted_free_slots();
        Ok(())
    }

    pub(crate) fn publish_transaction_header(&mut self, cleanup_sequence: u64) -> Result<()> {
        let generation = self.header_generation.checked_add(1).ok_or_else(|| {
            Error::SecurityLimitExceeded("header generation exhausted".to_string())
        })?;
        // Once header publication begins, a write/sync error is ambiguous: the
        // inactive slot may already be durable even though the call returned an
        // error. Keep this handle poisoned until the complete publication has
        // succeeded so commit cannot roll back to stale in-memory metadata.
        self.poisoned = Some(
            "header publication was interrupted; reopen the lockbox before continuing".to_string(),
        );
        let mut publication = Publication {
            generation,
            commit_root_offset: self.commit_root_offset,
            sequence: self.sequence,
            key_directory_offset: self.key_directory.primary_offset,
            key_directory_mirror_offset: self.key_directory.mirror_offset,
            lockbox_id: self.lockbox_id,
            commit_auth_offset: self.commit_auth_offset,
            cleanup_sequence,
            cleanup_completed_ranges: self.cleanup_completed_ranges,
            cleanup_completed_pages: self.cleanup_completed_pages,
            cleanup_completed_bytes: self.cleanup_completed_bytes,
            metadata_auth_tag: [0; 24],
        };
        let message = header_v2::metadata_auth_message(publication);
        publication.metadata_auth_tag = self
            .key
            .with_bytes(|key| crate::crypto::metadata_auth_tag(key, &message))?;
        let header_slot = self.header_slot;
        let next_slot = publish_header(&mut self.storage, header_slot, publication)?;
        self.header_slot = next_slot;
        self.header_generation = generation;
        self.cleanup_sequence = cleanup_sequence;
        self.poisoned = None;
        Ok(())
    }

    fn write_key_directory_mirrors_if_dirty(&mut self) -> Result<()> {
        if !self.key_directory.dirty {
            return Ok(());
        }
        let old_offsets = self.key_directory.offsets();
        if self.key_slots.is_empty() {
            self.key_directory.clear();
            for offset in old_offsets {
                self.zero_key_directory_page(offset)?;
            }
            return Ok(());
        }
        let key_slots = self.key_slots.clone();
        let mut offsets = [0u64; 2];
        for copy_index in 0..offsets.len() {
            let key_directory = encode_key_directory(
                &key_slots,
                self.lockbox_id,
                self.key_directory.generation,
                copy_index as u32,
            )?;
            let object =
                PageObject::new(PageObjectKind::KeyDirectory, self.sequence, key_directory);
            let page_size = page_size_for_encoded_objects(std::slice::from_ref(&object))? as u64;
            let page_offset = if copy_index == 0 {
                self.allocate_page_offset(page_size)?
            } else {
                self.allocate_key_directory_mirror_offset(page_size, offsets[0])?
            };
            self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
            offsets[copy_index] = page_offset;
        }
        self.key_directory.publish(offsets);
        for offset in old_offsets {
            self.zero_key_directory_page(offset)?;
        }
        Ok(())
    }

    fn allocate_key_directory_mirror_offset(
        &mut self,
        page_size: u64,
        primary_offset: u64,
    ) -> Result<u64> {
        if let Some(slot) = self.free_space.allocate_away_from(
            page_size,
            primary_offset,
            KEY_DIRECTORY_MIRROR_PREFERRED_MIN_DISTANCE,
        ) {
            return Ok(slot.offset);
        }
        self.next_append_page_offset()
    }

    fn zero_key_directory_page(&mut self, offset: u64) -> Result<()> {
        if offset != 0 {
            let page_len = self.page_len_at(offset)?;
            self.zero_page_and_free(crate::free_slot::FreeSlot {
                offset,
                len: page_len,
            })?;
        }
        Ok(())
    }

    fn page_len_at(&self, offset: u64) -> Result<u64> {
        let header = self.storage.read_at(offset, crate::page::PAGE_HEADER_LEN)?;
        if header.get(0..8) != Some(crate::page::PAGE_MAGIC.as_slice()) {
            return Err(Error::CorruptRecord);
        }
        let header_len = read_u32_le(&header[12..16])? as usize;
        let stored_body_len = read_u32_le(&header[44..48])? as usize;
        let stored_len = header_len
            .checked_add(stored_body_len)
            .ok_or(Error::CorruptRecord)?;
        Ok(
            crate::page::page_size_for_stored_len(stored_len, crate::page::DEFAULT_DATA_PAGE_BYTES)?
                as u64,
        )
    }

    fn commit_toc_btree(&mut self) -> Result<u64> {
        if self.toc_tree.root.is_some() && self.toc_tree.is_clean() {
            return Ok(self.toc_tree.root_offset);
        }
        let root = if self.toc_tree.leaves.is_empty() {
            self.rebuild_toc_btree()?
        } else {
            self.write_incremental_toc_btree()?
        };
        self.toc_tree.dirty_keys.clear();
        Ok(root)
    }

    fn rebuild_toc_btree(&mut self) -> Result<u64> {
        let entries = self.toc_entries.values().cloned().collect::<Vec<_>>();
        if entries.is_empty() {
            let offset = self.write_toc_leaf(&[])?;
            let leaf = TocLeaf {
                offset,
                entries: Vec::new(),
            };
            self.toc_tree.root = Some(TocTreeNode::Leaf(leaf.clone()));
            self.toc_tree.leaves = vec![leaf];
            return Ok(offset);
        }

        let mut leaves = Vec::new();
        for chunk in toc_leaf_groups(&entries)? {
            let offset = self.write_toc_leaf(chunk)?;
            leaves.push(TocLeaf {
                offset,
                entries: chunk.to_vec(),
            });
        }
        let root_node = self.write_toc_tree_for_leaves(&leaves)?;
        let root = root_node.offset();
        self.toc_tree.replace_topology(root_node, leaves);
        Ok(root)
    }

    fn write_incremental_toc_btree(&mut self) -> Result<u64> {
        let dirty = std::mem::take(&mut self.toc_tree.dirty_keys);
        let all_entries = self.toc_entries.values().cloned().collect::<Vec<_>>();
        let old_leaves = std::mem::take(&mut self.toc_tree.leaves);
        let rewrites =
            IncrementalBTree::new(&all_entries, &old_leaves, &dirty).plan(same_leaf_entries);
        let mut rebuilt_leaves = Vec::new();
        for rewrite in rewrites {
            match rewrite {
                LeafRewrite::Reuse(leaf) => rebuilt_leaves.push(leaf),
                LeafRewrite::Write(entries) => {
                    for chunk in toc_leaf_groups(&entries)? {
                        let offset = self.write_toc_leaf(chunk)?;
                        rebuilt_leaves.push(TocLeaf {
                            offset,
                            entries: chunk.to_vec(),
                        });
                    }
                }
            }
        }
        if rebuilt_leaves.is_empty() {
            let offset = self.write_toc_leaf(&[])?;
            rebuilt_leaves.push(TocLeaf {
                offset,
                entries: Vec::new(),
            });
        }
        rebuilt_leaves.sort_by(|left, right| leaf_first_path(left).cmp(leaf_first_path(right)));
        let root_node = if leaf_directory_is_compatible(&old_leaves, &rebuilt_leaves) {
            let old_root = self.toc_tree.root.take().ok_or(Error::CorruptRecord)?;
            self.rewrite_compatible_toc_tree(old_root, &rebuilt_leaves)?
        } else {
            self.write_toc_tree_for_leaves(&rebuilt_leaves)?
        };
        let root = root_node.offset();
        self.toc_tree.replace_topology(root_node, rebuilt_leaves);
        Ok(root)
    }

    fn write_toc_tree_for_leaves(&mut self, leaves: &[TocLeaf]) -> Result<TocTreeNode> {
        if leaves.len() == 1 {
            return Ok(TocTreeNode::Leaf(leaves[0].clone()));
        }
        let mut level = leaves
            .iter()
            .cloned()
            .map(TocTreeNode::Leaf)
            .collect::<Vec<_>>();

        while level.len() > 1 {
            let mut next_level = Vec::new();
            let mut child_cursor = 0usize;
            let children = level
                .iter()
                .map(|node| {
                    Ok(TocChild {
                        first_path: crate::LockboxPath::from_stored(node.first_path(), false)?,
                        offset: node.offset(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            for chunk in toc_child_groups(&children)? {
                let offset = self.write_toc_internal(chunk)?;
                let start = child_cursor;
                let end = start + chunk.len();
                child_cursor = end;
                let child_nodes = level[start..end].to_vec();
                next_level.push(TocTreeNode::Internal(TocInternal {
                    offset,
                    children: child_nodes,
                }));
            }
            level = next_level;
        }

        Ok(level.remove(0))
    }

    fn rewrite_compatible_toc_tree(
        &mut self,
        node: TocTreeNode,
        new_leaves: &[TocLeaf],
    ) -> Result<TocTreeNode> {
        match node {
            TocTreeNode::Leaf(old_leaf) => {
                let Some(new_leaf) = new_leaves
                    .iter()
                    .find(|leaf| leaf_first_path(leaf) == leaf_first_path(&old_leaf))
                    .cloned()
                else {
                    return Err(Error::CorruptRecord);
                };
                Ok(TocTreeNode::Leaf(new_leaf))
            }
            TocTreeNode::Internal(old_internal) => {
                let old_offset = old_internal.offset;
                let mut changed = false;
                let mut children = Vec::with_capacity(old_internal.children.len());
                for child in old_internal.children {
                    let child_offset = child.offset();
                    let child_first_path = child.first_path().to_string();
                    let rewritten = self.rewrite_compatible_toc_tree(child, new_leaves)?;
                    if rewritten.offset() != child_offset
                        || rewritten.first_path() != child_first_path.as_str()
                    {
                        changed = true;
                    }
                    children.push(rewritten);
                }
                if !changed {
                    return Ok(TocTreeNode::Internal(TocInternal {
                        offset: old_offset,
                        children,
                    }));
                }
                let toc_children = children
                    .iter()
                    .map(|child| {
                        Ok(TocChild {
                            first_path: crate::LockboxPath::from_stored(child.first_path(), false)?,
                            offset: child.offset(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let offset = self.write_toc_internal(&toc_children)?;
                Ok(TocTreeNode::Internal(TocInternal { offset, children }))
            }
        }
    }

    fn write_toc_leaf(&mut self, entries: &[crate::toc_entry::TocEntry]) -> Result<u64> {
        let payload = encode_toc_leaf(entries)?;
        self.sequence += 1;
        self.append_toc_page(PageObjectKind::TocLeaf, payload)
    }

    fn write_toc_internal(&mut self, children: &[TocChild]) -> Result<u64> {
        let payload = encode_toc_internal(children)?;
        self.sequence += 1;
        self.append_toc_page(PageObjectKind::TocInternal, payload)
    }

    fn append_toc_page(&mut self, kind: PageObjectKind, payload: Vec<u8>) -> Result<u64> {
        let object = PageObject::new(kind, self.sequence, payload);
        let page_offset = self.allocate_page_offset(page_size_for_encoded_objects(
            std::slice::from_ref(&object),
        )? as u64)?;
        self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
        Ok(page_offset)
    }

    fn write_free_index(&mut self) -> Result<u64> {
        let slots = self.free_space.slots_by_offset();
        self.write_free_index_slots(&slots)
    }

    fn write_post_cleanup_free_index(&mut self) -> Result<u64> {
        if self.redacted_free_slots.is_empty() {
            return Ok(0);
        }
        let mut post_cleanup = self.free_space.clone();
        for slot in self.redacted_free_slots.iter().copied() {
            post_cleanup.add(slot);
        }
        let slots = post_cleanup.slots_by_offset();
        self.write_free_index_slots(&slots)
    }

    fn write_free_index_slots(&mut self, slots: &[crate::free_slot::FreeSlot]) -> Result<u64> {
        if slots.is_empty() {
            return Ok(0);
        }
        self.sequence += 1;
        let mut level = Vec::new();
        for group in free_index_leaf_groups(slots) {
            let offset = self.write_free_index_page(
                PageObjectKind::FreeIndexLeaf,
                encode_free_index_leaf(group),
            )?;
            level.push(FreeIndexChild {
                first_offset: group[0].offset,
                offset,
            });
        }
        while level.len() > 1 {
            let mut next = Vec::new();
            for group in free_index_child_groups(&level) {
                let offset = self.write_free_index_page(
                    PageObjectKind::FreeIndexInternal,
                    encode_free_index_internal(group),
                )?;
                next.push(FreeIndexChild {
                    first_offset: group[0].first_offset,
                    offset,
                });
            }
            level = next;
        }
        Ok(level[0].offset)
    }

    fn write_free_index_page(&mut self, kind: PageObjectKind, payload: Vec<u8>) -> Result<u64> {
        let page_offset = self.next_append_page_offset()?;
        let object = PageObject::new(kind, self.sequence, payload);
        self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
        Ok(page_offset)
    }

    fn append_commit_root_page(&mut self, payload: Vec<u8>) -> Result<u64> {
        let page_offset = self.next_append_page_offset()?;
        let object = PageObject::new(PageObjectKind::CommitRoot, self.sequence, payload);
        self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
        Ok(page_offset)
    }

    fn append_commit_auth_page(&mut self, payload: Vec<u8>) -> Result<u64> {
        let page_offset = self.next_append_page_offset()?;
        let object = PageObject::new(PageObjectKind::CommitAuth, self.sequence, payload);
        self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
        Ok(page_offset)
    }

    fn append_redaction_manifest_pages(
        &mut self,
        ranges: &[crate::free_slot::FreeSlot],
    ) -> Result<(u64, u32, u64)> {
        let ranges = bounded_ranges(ranges)?;
        if ranges.is_empty() {
            return Ok((0, 0, 0));
        }
        let total_range_count = u32::try_from(ranges.len()).map_err(|_| {
            Error::SecurityLimitExceeded("redaction range count exceeds u32".to_string())
        })?;
        let total_bytes = ranges.iter().try_fold(0u64, |total, range| {
            total.checked_add(range.len).ok_or_else(|| {
                Error::SecurityLimitExceeded("redaction byte count overflow".to_string())
            })
        })?;
        if total_bytes > crate::file_format::redaction_manifest::MAX_REDACTION_TOTAL_BYTES {
            return Err(Error::SecurityLimitExceeded(format!(
                "redaction transaction exceeds the {} byte limit",
                crate::file_format::redaction_manifest::MAX_REDACTION_TOTAL_BYTES
            )));
        }
        let mut next_page_offset = 0;
        for chunk in ranges.chunks(RANGES_PER_PAGE).rev() {
            let payload = encode_redaction_manifest_page(&RedactionManifestPage {
                transaction_sequence: self.sequence,
                next_page_offset,
                total_range_count,
                ranges: chunk.to_vec(),
            })?;
            let page_offset = self.next_append_page_offset()?;
            let object = PageObject::new(PageObjectKind::RedactionManifest, self.sequence, payload);
            self.write_decoded_page_at(page_offset, self.sequence, vec![object])?;
            next_page_offset = page_offset;
        }
        Ok((next_page_offset, total_range_count, total_bytes))
    }
}

pub(crate) struct CommitRollback(StagedLockboxState);

impl CommitRollback {
    pub(crate) fn capture(lockbox: &Lockbox) -> Self {
        Self(lockbox.staged.clone())
    }

    pub(crate) fn restore(self, lockbox: &mut Lockbox) {
        lockbox.staged = self.0;
        lockbox.page_manager.borrow_mut().clear();
    }
}

fn same_leaf_entries(
    old: &[crate::toc_entry::TocEntry],
    new: &[crate::toc_entry::TocEntry],
) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(old, new)| {
            old.path == new.path
                && old.len == new.len
                && old.record_offset == new.record_offset
                && old.record_len == new.record_len
                && old.record_object_id == new.record_object_id
                && old.deleted == new.deleted
                && old.node_kind == new.node_kind
                && old.permissions == new.permissions
                && old.chunks == new.chunks
        })
}

fn leaf_first_path(leaf: &TocLeaf) -> &str {
    leaf.entries
        .first()
        .map(|entry| entry.path.as_str())
        .unwrap_or("")
}

fn leaf_directory_is_compatible(old: &[TocLeaf], new: &[TocLeaf]) -> bool {
    old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| leaf_first_path(old) == leaf_first_path(new))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_FILE_PERMISSIONS;
    use crate::lockbox_path::LockboxPath;
    use crate::node_kind::NodeKind;
    use crate::toc_entry::TocEntry;
    use crate::{Error, LockboxOpen, LockboxProtection, OwnerSigningKeyPair, SecretString};
    use std::path::PathBuf;

    fn p(path: impl AsRef<str>) -> LockboxPath {
        LockboxPath::new(path).unwrap()
    }

    fn add_file<State>(
        lb: &mut Lockbox<State>,
        path: &LockboxPath,
        data: &[u8],
        replace: bool,
    ) -> crate::Result<()>
    where
        State: crate::WritableLockboxState,
    {
        lb.create_parent_dirs_for(path)?;
        Lockbox::add_file(lb, path, data, replace)
    }

    fn unique_path(label: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/test-tmp")
            .join(format!(
                "lockbox-core-{label}-{}-{}.lbox",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(name: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let previous = std::env::var_os(name);
            std::env::set_var(name, value);
            Self { name, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.name, value),
                None => std::env::remove_var(self.name),
            }
        }
    }

    #[test]
    fn file_backed_lockbox_write_lock_blocks_other_threads() {
        let _timeout_guard = EnvVarGuard::set("LOCKBOX_LOCK_TIMEOUT_MS", "150");
        let path = unique_path("write-lock-thread");
        let password = SecretString::try_from_bytes(b"password".to_vec()).unwrap();
        let signing_key = OwnerSigningKeyPair::generate().unwrap();
        let lockbox =
            Lockbox::create_file(&path, LockboxProtection::Password(&password), &signing_key)
                .unwrap();
        let blocked_path = path.clone();
        let blocked = std::thread::spawn(move || {
            let password = SecretString::try_from_bytes(b"password".to_vec()).unwrap();
            let signing_key = OwnerSigningKeyPair::generate().unwrap();
            Lockbox::open_for_write(
                &blocked_path,
                LockboxOpen::Password(&password),
                &signing_key,
            )
        })
        .join()
        .unwrap();
        assert!(matches!(blocked, Err(Error::LockUnavailable(_))));

        drop(lockbox);
        let reopened =
            Lockbox::open_for_write(&path, LockboxOpen::Password(&password), &signing_key).unwrap();
        drop(reopened);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn file_backed_write_locks_are_scoped_to_each_lockbox() {
        let path_a = unique_path("write-lock-a");
        let path_b = unique_path("write-lock-b");
        let password = SecretString::try_from_bytes(b"password".to_vec()).unwrap();
        let signing_key = OwnerSigningKeyPair::generate().unwrap();
        let lockbox_a = Lockbox::create_file(
            &path_a,
            LockboxProtection::Password(&password),
            &signing_key,
        )
        .unwrap();
        let lockbox_b = Lockbox::create_file(
            &path_b,
            LockboxProtection::Password(&password),
            &signing_key,
        )
        .unwrap();

        drop(lockbox_b);
        drop(lockbox_a);
        let _ = std::fs::remove_file(&path_a);
        let _ = std::fs::remove_file(&path_b);
    }

    #[test]
    fn compatible_toc_update_rewrites_only_changed_leaf_and_ancestors() {
        let mut lb = Lockbox::create("secret");
        for entry in synthetic_toc_entries(150_000) {
            lb.toc_entries.insert(entry.path.clone(), entry);
        }
        lb.commit().unwrap();
        let len_after_create = lb.to_bytes().len();
        let root_after_create = lb.toc_tree.root_offset;
        let old_leaf_offsets = lb
            .toc_tree
            .leaves
            .iter()
            .map(|leaf| leaf.offset)
            .collect::<Vec<_>>();
        assert!(old_leaf_offsets.len() > 1);

        let path = "/toc-cow/file-00001.txt";
        let entry = lb.toc_entries.get_mut(path).unwrap();
        entry.record_offset += 1;
        lb.mark_toc_dirty(&p(path));
        lb.commit().unwrap();
        let new_leaf_offsets = lb
            .toc_tree
            .leaves
            .iter()
            .map(|leaf| leaf.offset)
            .collect::<Vec<_>>();

        assert_ne!(lb.toc_tree.root_offset, root_after_create);
        assert_eq!(old_leaf_offsets.len(), new_leaf_offsets.len());
        assert!(
            old_leaf_offsets
                .iter()
                .zip(&new_leaf_offsets)
                .filter(|(old, new)| old != new)
                .count()
                <= 1
        );
        assert!(lb.to_bytes().len() > len_after_create);
    }

    #[test]
    fn incremental_toc_insert_before_first_leaf_does_not_duplicate_entries() {
        let mut lb = Lockbox::create("secret");
        let existing = synthetic_toc_entries(1).remove(0);
        lb.toc_entries.insert(existing.path.clone(), existing);
        lb.commit().unwrap();

        let mut inserted = synthetic_toc_entries(1).remove(0);
        inserted.path = p("/before-first.txt");
        lb.toc_entries.insert(inserted.path.clone(), inserted);
        lb.mark_toc_dirty(&p("/before-first.txt"));
        lb.commit().unwrap();

        let persisted_paths = lb
            .toc_tree
            .leaves
            .iter()
            .flat_map(|leaf| leaf.entries.iter().map(|entry| entry.path.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            persisted_paths,
            vec!["/before-first.txt", "/toc-cow/file-00000.txt"]
        );

        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert_eq!(reopened.toc_entries.len(), 2);
    }

    #[test]
    fn noop_commit_reuses_existing_toc_root() {
        let mut lb = Lockbox::create("secret");
        for entry in synthetic_toc_entries(100) {
            lb.toc_entries.insert(entry.path.clone(), entry);
        }
        lb.commit().unwrap();
        let root = lb.toc_tree.root_offset;
        let len = lb.to_bytes().len();

        lb.commit().unwrap();

        assert_eq!(lb.toc_tree.root_offset, root);
        assert_eq!(lb.to_bytes().len(), len);
    }

    #[test]
    fn header_points_to_commit_root_that_points_to_toc_root() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        lb.commit().unwrap();

        let bytes = lb.to_bytes();
        let header = crate::file_format::read_header(&bytes).unwrap();
        let header_root_offset = header.commit_root_offset;
        assert_eq!(header_root_offset, lb.commit_root_offset);
        assert_eq!(header.commit_auth_offset, lb.commit_auth_offset);
        assert_ne!(header.commit_auth_offset, 0);
        assert_ne!(header_root_offset, lb.toc_tree.root_offset);

        let page = crate::page::page_decode_slice(&bytes, header_root_offset as usize).unwrap();
        let decoded = lb
            .key
            .with_bytes(|key| crate::page::decode_page(page, lb.lockbox_id, key))
            .unwrap()
            .unwrap();
        let commit_object = decoded
            .objects
            .iter()
            .find(|object| object.kind == PageObjectKind::CommitRoot)
            .unwrap();
        let commit_root = commit_object
            .with_payload(crate::commit_root::decode_commit_root)
            .unwrap()
            .unwrap();
        assert_eq!(commit_root.toc_root_offset, lb.toc_tree.root_offset);
    }

    #[test]
    fn signed_commit_round_trips_after_reopen() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        lb.commit().unwrap();
        let first_auth = lb.commit_auth_offset;

        let mut reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        add_file(&mut reopened, &p("/docs/b.txt"), b"bravo", false).unwrap();
        reopened.commit().unwrap();

        assert_ne!(reopened.commit_auth_offset, first_auth);
        assert_eq!(
            reopened.read_file_range(&p("/docs/a.txt"), 0, 100).unwrap(),
            b"alpha"
        );
        assert_eq!(
            reopened.read_file_range(&p("/docs/b.txt"), 0, 100).unwrap(),
            b"bravo"
        );
    }

    #[test]
    fn open_rejects_tampered_signed_commit_auth_page() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        lb.commit().unwrap();

        let mut bytes = lb.to_bytes();
        let auth_offset = crate::file_format::read_header(&bytes)
            .unwrap()
            .commit_auth_offset as usize;
        bytes[auth_offset + crate::page::PAGE_HEADER_LEN + 4] ^= 0x01;

        assert!(Lockbox::open_bytes_with_key(bytes, "secret").is_err());
    }

    #[test]
    fn open_rejects_tampered_signed_commit_root_page() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        lb.commit().unwrap();

        let mut bytes = lb.to_bytes();
        let root_offset = crate::file_format::read_header(&bytes)
            .unwrap()
            .commit_root_offset as usize;
        bytes[root_offset + crate::page::PAGE_HEADER_LEN + 4] ^= 0x01;

        assert!(Lockbox::open_bytes_with_key(bytes, "secret").is_err());
    }

    #[test]
    fn contact_opened_lockbox_cannot_commit_changes() {
        let contact = crate::ContactKeyPair::generate().unwrap();
        let mut lb = Lockbox::create_with_contact(&contact.public_key()).unwrap();
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        lb.commit().unwrap();

        let mut opened = Lockbox::open_with_contact(lb.to_bytes(), &contact).unwrap();
        add_file(&mut opened, &p("/docs/b.txt"), b"bravo", false).unwrap();

        assert!(matches!(
            opened.commit(),
            Err(crate::Error::InvalidOperation(message))
                if message.contains("contact-opened lockboxes are read-only")
        ));
    }

    #[test]
    fn committed_toc_is_live_only_after_delete() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/a.txt"), b"alpha", false).unwrap();
        add_file(&mut lb, &p("/docs/b.txt"), b"bravo", false).unwrap();
        lb.commit().unwrap();

        lb.delete(&p("/docs/a.txt")).unwrap();
        lb.commit().unwrap();

        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert!(!reopened.toc_entries.contains_key("/docs/a.txt"));
        assert!(reopened.toc_entries.contains_key("/docs/b.txt"));
        assert!(reopened
            .toc_tree
            .leaves
            .iter()
            .flat_map(|leaf| leaf.entries.iter())
            .all(|entry| !entry.deleted && entry.path != "/docs/a.txt"));
    }

    #[test]
    fn commit_root_restores_persisted_free_index_on_open() {
        let mut lb = Lockbox::create("secret");
        let mut state = 0x1234_5678u64;
        let data = (0..(6 * 1024 * 1024))
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                (state >> 32) as u8
            })
            .collect::<Vec<_>>();
        for index in 0..6 {
            add_file(
                &mut lb,
                &p(format!("/docs/remove-{index}.bin")),
                &data,
                false,
            )
            .unwrap();
        }
        lb.commit().unwrap();
        for index in 0..6 {
            lb.delete(&p(format!("/docs/remove-{index}.bin"))).unwrap();
        }
        lb.commit().unwrap();

        assert!(lb.free_index_offset > 0);
        assert!(!lb.free_space.slots_by_offset().is_empty());
        let bytes = lb.to_bytes();
        assert_eq!(
            &bytes[lb.free_index_offset as usize..lb.free_index_offset as usize + 8],
            crate::page::PAGE_MAGIC
        );
        let reopened = Lockbox::open_bytes_with_key(bytes, "secret").unwrap();
        assert!(!reopened.free_space.slots_by_offset().is_empty());
    }

    #[test]
    fn free_index_overflow_writes_internal_nodes_and_reopens() {
        let mut lb = Lockbox::create("secret");
        let mut state = 0x1234_5678u64;
        let data = (0..(2 * 1024 * 1024))
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                (state >> 32) as u8
            })
            .collect::<Vec<_>>();
        for index in 0..((crate::free_index::FREE_INDEX_LEAF_SLOT_CAPACITY + 3) * 2) {
            add_file(
                &mut lb,
                &p(format!("/docs/free-index-{index}.bin")),
                &data,
                false,
            )
            .unwrap();
        }
        lb.commit().unwrap();
        for index in (0..((crate::free_index::FREE_INDEX_LEAF_SLOT_CAPACITY + 3) * 2)).step_by(2) {
            lb.delete(&p(format!("/docs/free-index-{index}.bin")))
                .unwrap();
        }
        lb.commit().unwrap();
        assert!(lb.free_index_offset > 0);
        let bytes = lb.to_bytes();
        let root_page =
            crate::page::page_decode_slice(&bytes, lb.free_index_offset as usize).unwrap();
        let decoded = lb
            .key
            .with_bytes(|key| crate::page::decode_page(root_page, lb.lockbox_id, key))
            .unwrap()
            .unwrap();
        assert!(decoded
            .objects
            .iter()
            .any(|object| object.kind == PageObjectKind::FreeIndexInternal));

        let reopened = Lockbox::open_bytes_with_key(bytes, "secret").unwrap();
        assert!(
            reopened.free_space.slots_by_offset().len()
                > crate::free_index::FREE_INDEX_LEAF_SLOT_CAPACITY
        );
    }

    #[test]
    fn failed_commit_after_partial_appends_reopens_previous_commit() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/old.txt"), b"old", false).unwrap();
        lb.commit().unwrap();

        add_file(&mut lb, &p("/docs/new.txt"), b"new", false).unwrap();
        lb.storage.fail_memory_append_after_successes(1);
        assert!(matches!(lb.commit(), Err(Error::Io(_))));

        assert_eq!(lb.get_file(&p("/docs/new.txt")).unwrap(), b"new");
        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert_eq!(reopened.get_file(&p("/docs/old.txt")).unwrap(), b"old");
        assert!(matches!(
            reopened.get_file(&p("/docs/new.txt")),
            Err(Error::NotFound(_))
        ));

        lb.commit().unwrap();
        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert_eq!(reopened.get_file(&p("/docs/new.txt")).unwrap(), b"new");
    }

    #[test]
    fn failed_commit_header_publish_reopens_previous_commit() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/old.txt"), b"old", false).unwrap();
        lb.commit().unwrap();

        add_file(&mut lb, &p("/docs/new.txt"), b"new", false).unwrap();
        lb.storage.fail_memory_next_write_at(0);
        assert!(matches!(lb.commit(), Err(Error::Io(_))));

        assert_eq!(lb.get_file(&p("/docs/new.txt")).unwrap(), b"new");
        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert_eq!(reopened.get_file(&p("/docs/old.txt")).unwrap(), b"old");
        assert!(matches!(
            reopened.get_file(&p("/docs/new.txt")),
            Err(Error::NotFound(_))
        ));

        assert!(matches!(lb.commit(), Err(Error::InvalidOperation(_))));
    }

    #[test]
    fn ambiguous_header_sync_failure_poisons_handle_and_opens_published_commit() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/new.txt"), b"new", false).unwrap();

        // The first sync makes pages and commit metadata durable. The second
        // follows the inactive-slot header write, so failure is ambiguous even
        // though the memory backend retains the written header bytes.
        lb.storage.fail_memory_sync_after_successes(1);
        assert!(matches!(lb.commit(), Err(Error::Io(_))));
        assert!(matches!(lb.commit(), Err(Error::InvalidOperation(_))));

        let reopened = Lockbox::open_bytes_with_key(lb.to_bytes(), "secret").unwrap();
        assert_eq!(reopened.get_file(&p("/docs/new.txt")).unwrap(), b"new");
    }

    #[test]
    fn interrupted_cleanup_requires_explicit_recovery_and_is_idempotent() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        add_file(&mut lb, &p("/docs/keep.txt"), b"keep", false).unwrap();
        lb.commit().unwrap();
        let removed_slot = {
            let entry = lb.toc_entries.get(&p("/docs/remove.txt")).unwrap();
            crate::free_slot::FreeSlot {
                offset: entry.record_offset,
                len: entry.record_len,
            }
        };

        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_slot.offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));
        let interrupted = lb.to_bytes();
        assert!(matches!(
            Lockbox::open_bytes_with_key(interrupted.clone(), "secret"),
            Err(Error::RecoveryRequired { .. })
        ));

        let key = crate::SecretVec::try_from_slice(b"secret").unwrap();
        let mut recovering = Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(interrupted),
            key,
            crate::LockboxOptions::default(),
            true,
        )
        .unwrap();
        assert!(!recovering.free_space.overlaps(removed_slot));
        let mut post_cleanup_space = crate::free_slot::FreeSpace::default();
        post_cleanup_space.replace_slots(
            recovering
                .read_free_index_slots(recovering.post_cleanup_free_index_offset, 0)
                .unwrap(),
        );
        assert!(post_cleanup_space.contains(removed_slot));
        let mut updates = Vec::new();
        recovering
            .cleanup_published_redactions(|update| updates.push(update))
            .unwrap();
        // Repeating cleanup before sealing is safe after a crash at this boundary.
        recovering.cleanup_published_redactions(|_| {}).unwrap();
        recovering
            .publish_transaction_header(recovering.sequence)
            .unwrap();
        let recovered = recovering.to_bytes();
        assert!(recovered
            [removed_slot.offset as usize..(removed_slot.offset + removed_slot.len) as usize]
            .iter()
            .all(|byte| *byte == 0));
        assert_eq!(
            updates.last().unwrap().completed_ranges,
            updates.last().unwrap().total_ranges
        );

        let reopened = Lockbox::open_bytes_with_key(recovered, "secret").unwrap();
        assert!(reopened.free_space.contains(removed_slot));
        assert!(matches!(
            reopened.get_file(&p("/docs/remove.txt")),
            Err(Error::NotFound(_))
        ));
        assert_eq!(reopened.get_file(&p("/docs/keep.txt")).unwrap(), b"keep");
    }

    #[test]
    fn recovery_rejects_a_corrupt_manifest_without_zeroing_untrusted_ranges() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        lb.commit().unwrap();
        let removed_offset = lb
            .toc_entries
            .get(&p("/docs/remove.txt"))
            .unwrap()
            .record_offset;

        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));
        let manifest_offset = usize::try_from(lb.redaction_manifest_offset).unwrap();
        let mut damaged = lb.to_bytes();
        damaged[manifest_offset + crate::page::PAGE_HEADER_LEN + 8] ^= 0x55;

        let key = crate::SecretVec::try_from_slice(b"secret").unwrap();
        let mut recovering = Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(damaged),
            key,
            crate::LockboxOptions::default(),
            true,
        )
        .unwrap();
        assert!(recovering.cleanup_published_redactions(|_| {}).is_err());
    }

    #[test]
    fn recovery_rejects_inconsistent_durable_checkpoint_counters() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        lb.commit().unwrap();
        let removed_offset = lb
            .toc_entries
            .get(&p("/docs/remove.txt"))
            .unwrap()
            .record_offset;

        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));
        lb.cleanup_completed_pages = 1;
        lb.cleanup_completed_ranges = 0;
        lb.cleanup_completed_bytes = 0;
        lb.publish_transaction_header(lb.cleanup_sequence).unwrap();

        let key = crate::SecretVec::try_from_slice(b"secret").unwrap();
        let mut recovering = Lockbox::open_storage_with_secret_key_mode(
            crate::storage::StorageBackend::memory(lb.to_bytes()),
            key,
            crate::LockboxOptions::default(),
            true,
        )
        .unwrap();
        assert!(matches!(
            recovering.cleanup_published_redactions(|_| {}),
            Err(Error::CorruptHeader)
        ));
    }

    #[test]
    fn public_recovery_can_inspect_cancel_and_resume_from_a_durable_checkpoint() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        lb.commit().unwrap();
        let removed = lb.toc_entries.get(&p("/docs/remove.txt")).unwrap();
        let removed_offset = removed.record_offset;
        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));

        let path = std::env::temp_dir().join(format!(
            "revault-transaction-recovery-{}-{}.lbox",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, lb.to_bytes()).unwrap();
        let open =
            || crate::LockboxOpen::ContentKey(crate::SecretVec::try_from_slice(b"secret").unwrap());

        let initial = Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .unwrap();
        assert_eq!(initial.completed_pages, 0);
        let cancelled = Lockbox::recover_transaction_controlled(&path, open(), |_| {
            crate::TransactionRecoveryControl::Cancel
        })
        .unwrap();
        let crate::TransactionRecoveryOutcome::Cancelled(checkpoint) = cancelled else {
            panic!("expected a durable cancellation checkpoint");
        };
        assert_eq!(checkpoint.completed_pages, checkpoint.page_count);
        assert!(matches!(
            Lockbox::open(&path, open()),
            Err(Error::RecoveryRequired { .. })
        ));

        let resumed = Lockbox::recover_transaction_controlled(&path, open(), |_| {
            crate::TransactionRecoveryControl::Continue
        })
        .unwrap();
        assert_eq!(resumed, crate::TransactionRecoveryOutcome::Complete);
        assert!(Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .is_none());
        assert!(matches!(
            Lockbox::open(&path, open())
                .unwrap()
                .get_file(&p("/docs/remove.txt")),
            Err(Error::NotFound(_))
        ));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(crate::lock_path_for(&path));
    }

    #[test]
    fn recovery_resumes_after_a_progress_callback_panics() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        lb.commit().unwrap();
        let removed_offset = lb
            .toc_entries
            .get(&p("/docs/remove.txt"))
            .unwrap()
            .record_offset;
        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));

        let path = std::env::temp_dir().join(format!(
            "revault-transaction-recovery-panic-{}-{}.lbox",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, lb.to_bytes()).unwrap();
        let open =
            || crate::LockboxOpen::ContentKey(crate::SecretVec::try_from_slice(b"secret").unwrap());

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = Lockbox::recover_transaction_controlled(&path, open(), |_| {
                panic!("simulated termination after durable checkpoint")
            });
        }));
        assert!(panicked.is_err());
        let checkpoint = Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .unwrap();
        assert_eq!(checkpoint.completed_pages, checkpoint.page_count);
        assert!(Lockbox::recover_transaction(&path, open(), |_| {}).unwrap());
        assert!(Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .is_none());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(crate::lock_path_for(&path));
    }

    #[test]
    #[cfg(unix)]
    fn recovery_survives_a_real_subprocess_kill_after_checkpoint() {
        let mut lb = Lockbox::create("secret");
        add_file(&mut lb, &p("/docs/remove.txt"), b"remove me", false).unwrap();
        lb.commit().unwrap();
        let removed_offset = lb
            .toc_entries
            .get(&p("/docs/remove.txt"))
            .unwrap()
            .record_offset;
        lb.delete(&p("/docs/remove.txt")).unwrap();
        lb.storage.fail_memory_next_write_at(removed_offset);
        assert!(matches!(lb.commit(), Err(Error::RecoveryRequired { .. })));

        let path = std::env::temp_dir().join(format!(
            "revault-transaction-recovery-kill-{}-{}.lbox",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, lb.to_bytes()).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("lockbox::commit::tests::recovery_subprocess_kill_child")
            .arg("--nocapture")
            .env("REVAULT_RECOVERY_KILL_TEST_PATH", &path)
            .status()
            .unwrap();
        assert!(!status.success());

        let open =
            || crate::LockboxOpen::ContentKey(crate::SecretVec::try_from_slice(b"secret").unwrap());
        let checkpoint = Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .unwrap();
        assert_eq!(checkpoint.completed_pages, checkpoint.page_count);
        assert!(Lockbox::recover_transaction(&path, open(), |_| {}).unwrap());
        assert!(Lockbox::inspect_transaction_recovery(&path, open())
            .unwrap()
            .is_none());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(crate::lock_path_for(&path));
    }

    #[test]
    #[cfg(unix)]
    fn recovery_subprocess_kill_child() {
        let Some(path) = std::env::var_os("REVAULT_RECOVERY_KILL_TEST_PATH") else {
            return;
        };
        let key = crate::SecretVec::try_from_slice(b"secret").unwrap();
        let _ = Lockbox::recover_transaction_controlled(
            std::path::Path::new(&path),
            crate::LockboxOpen::ContentKey(key),
            |_| {
                // SAFETY: raising SIGKILL for this dedicated child process is
                // the behavior under test; no pointer or borrowed memory crosses FFI.
                unsafe { libc::kill(std::process::id() as i32, libc::SIGKILL) };
                crate::TransactionRecoveryControl::Continue
            },
        );
        panic!("SIGKILL unexpectedly returned");
    }

    #[test]
    fn unpublished_recipient_directory_cannot_unlock_archive() {
        let old_password = crate::SecretString::try_from_bytes(b"old password".to_vec()).unwrap();
        let new_password = crate::SecretString::try_from_bytes(b"new password".to_vec()).unwrap();
        let mut lb = Lockbox::create_with_password(&old_password).unwrap();
        add_file(&mut lb, &p("/docs/secret.txt"), b"secret", false).unwrap();
        lb.commit().unwrap();

        lb.add_password(&new_password).unwrap();
        let staged = lb.to_bytes();
        assert!(Lockbox::open_with_password(staged.clone(), &old_password).is_ok());
        assert!(matches!(
            Lockbox::open_with_password(staged, &new_password),
            Err(Error::InvalidKey)
        ));

        lb.commit().unwrap();
        assert!(Lockbox::open_with_password(lb.to_bytes(), &new_password).is_ok());
    }

    #[test]
    fn access_widening_compacts_abandoned_prepare_pages_out_of_the_archive() {
        let old_password = crate::SecretString::try_from_bytes(b"old password".to_vec()).unwrap();
        let new_password = crate::SecretString::try_from_bytes(b"new password".to_vec()).unwrap();
        let mut lb = Lockbox::create_with_password(&old_password).unwrap();
        let signing_key = lb.require_owner_signing_key().unwrap().try_clone().unwrap();
        add_file(&mut lb, &p("/docs/live.txt"), b"live", false).unwrap();
        lb.commit().unwrap();

        add_file(
            &mut lb,
            &p("/docs/abandoned.txt"),
            b"abandoned secret",
            false,
        )
        .unwrap();
        lb.storage.fail_memory_append_after_successes(1);
        assert!(matches!(lb.commit(), Err(Error::Io(_))));
        let prior_state = lb.to_bytes();

        let mut reopened = Lockbox::open_bytes_for_write(
            prior_state,
            crate::LockboxOpen::Password(&old_password),
            &signing_key,
        )
        .unwrap();
        reopened.add_password(&new_password).unwrap();
        reopened.commit().unwrap();
        let sanitized = reopened.to_bytes();
        let by_new = Lockbox::open_with_password(sanitized.clone(), &new_password).unwrap();
        assert_eq!(by_new.get_file(&p("/docs/live.txt")).unwrap(), b"live");
        assert!(matches!(
            by_new.get_file(&p("/docs/abandoned.txt")),
            Err(Error::NotFound(_))
        ));
        let report = by_new
            .key
            .with_bytes(|key| crate::RecoveryScanner::scan_bytes(sanitized, key))
            .unwrap();
        assert!(!report
            .intact_files
            .iter()
            .any(|entry| entry.path == "/docs/abandoned.txt"));
    }

    #[test]
    fn recipient_addition_requires_a_clean_separate_transaction() {
        let first = crate::SecretString::try_from_bytes(b"first password".to_vec()).unwrap();
        let second = crate::SecretString::try_from_bytes(b"second password".to_vec()).unwrap();
        let third = crate::SecretString::try_from_bytes(b"third password".to_vec()).unwrap();
        let mut lb = Lockbox::create_with_password(&first).unwrap();
        add_file(&mut lb, &p("/docs/original.txt"), b"original", false).unwrap();
        lb.commit().unwrap();

        add_file(&mut lb, &p("/docs/pending.txt"), b"pending", false).unwrap();
        assert!(matches!(
            lb.add_password(&second),
            Err(Error::InvalidOperation(_))
        ));
        lb.commit().unwrap();

        lb.add_password(&second).unwrap();
        assert!(matches!(
            lb.add_password(&third),
            Err(Error::InvalidOperation(_))
        ));
        lb.commit().unwrap();
        assert!(Lockbox::open_with_password(lb.to_bytes(), &second).is_ok());
    }

    fn synthetic_toc_entries(count: usize) -> Vec<TocEntry> {
        (0..count)
            .map(|i| TocEntry {
                path: LockboxPath::new(format!("/toc-cow/file-{i:05}.txt")).unwrap(),
                len: 1,
                record_offset: 1_000_000 + i as u64,
                record_len: 64,
                record_object_id: 1,
                deleted: false,
                node_kind: NodeKind::File,
                permissions: DEFAULT_FILE_PERMISSIONS,
                chunks: Vec::new(),
            })
            .collect()
    }
}
