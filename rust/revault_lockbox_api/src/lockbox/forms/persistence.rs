use super::*;

impl<State> Lockbox<State> {
    pub(crate) fn clone_all_form_state(
        &self,
    ) -> Result<(
        BTreeMap<String, FormDefinition>,
        BTreeMap<LockboxPath, FormRecord>,
    )> {
        self.ensure_forms_loaded()?;
        Ok((
            self.forms
                .definitions
                .borrow()
                .as_ref()
                .ok_or(Error::CorruptRecord)?
                .clone(),
            self.forms
                .records
                .borrow()
                .as_ref()
                .ok_or(Error::CorruptRecord)?
                .clone(),
        ))
    }

    pub(crate) fn set_form_definition_value(
        &mut self,
        key: String,
        definition: FormDefinition,
    ) -> Result<()> {
        self.ensure_forms_loaded()?;
        self.forms.put_definition_at(key, definition)
    }

    pub(crate) fn set_form_record_value(
        &mut self,
        path: LockboxPath,
        record: FormRecord,
    ) -> Result<()> {
        self.ensure_parent_directory(&path)?;
        self.ensure_forms_loaded()?;
        self.forms.put_record(path, record)
    }

    pub(crate) fn commit_form_tree(&mut self) -> Result<u64> {
        if !self.forms.dirty {
            return Ok(self.forms.tree.root_offset);
        }
        self.ensure_forms_loaded()?;
        let definitions = self
            .forms
            .definitions
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .clone();
        let records = self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .clone();
        if definitions.is_empty() && records.is_empty() {
            self.forms.tree.clear();
            self.forms.dirty = false;
            return Ok(0);
        }
        let root = if self.forms.tree.leaves.is_empty() {
            self.rebuild_form_btree(definitions, records)?
        } else {
            self.write_incremental_form_btree(definitions, records)?
        };
        self.forms.tree.dirty_keys.clear();
        self.forms.dirty = false;
        Ok(root)
    }

    pub(crate) fn stage_form_tree_redactions(&mut self) -> Result<()> {
        if !self.forms.dirty || self.forms.tree.root_offset == 0 {
            return Ok(());
        }
        let mut redactions = Vec::new();
        if !self.forms.tree.dirty_keys.is_empty() {
            let Some(root) = self.forms.tree.root.clone() else {
                self.collect_form_tree_redactions(self.forms.tree.root_offset, 0, &mut redactions)?;
                return self.write_form_redactions(redactions);
            };
            collect_dirty_form_tree_redactions_from_node(
                &root,
                &self.forms.tree.dirty_keys,
                &mut redactions,
            );
        } else {
            self.collect_form_tree_redactions(self.forms.tree.root_offset, 0, &mut redactions)?;
        }
        self.write_form_redactions(redactions)
    }

    fn write_form_redactions(&mut self, redactions: Vec<(u64, u64)>) -> Result<()> {
        for (offset, object_id) in redactions {
            self.sequence += 1;
            let payload = encode_form_leaf_secure(&[])?;
            let object = PageObject::new_secure(PageObjectKind::FormLeaf, object_id, payload);
            let page_size = page_size_for_objects(std::slice::from_ref(&object)) as u64;
            self.write_decoded_page_at(offset, self.sequence, vec![object])?;
            self.record_ref_counts.remove(&offset);
            self.redacted_free_slots.push(FreeSlot {
                offset,
                len: page_size,
            });
        }
        Ok(())
    }

    fn rebuild_form_btree(
        &mut self,
        definitions: BTreeMap<String, FormDefinition>,
        records: BTreeMap<LockboxPath, FormRecord>,
    ) -> Result<u64> {
        let entries = form_entries_from_maps(&definitions, &records);
        let mut leaves = Vec::new();
        for chunk in form_leaf_groups(&entries)? {
            let (offset, object_id) = self.write_form_leaf(chunk)?;
            leaves.push(FormLeaf {
                offset,
                object_id,
                entries: chunk.to_vec(),
            });
        }
        let root_node = self.write_form_tree_for_leaves(&leaves)?;
        let root = root_node.offset();
        self.forms.tree.replace_topology(root_node, leaves);
        Ok(root)
    }

    fn write_incremental_form_btree(
        &mut self,
        definitions: BTreeMap<String, FormDefinition>,
        records: BTreeMap<LockboxPath, FormRecord>,
    ) -> Result<u64> {
        let dirty = self.forms.tree.dirty_keys.clone();
        let all_entries = form_entries_from_maps(&definitions, &records);
        let old_leaves = std::mem::take(&mut self.forms.tree.leaves);
        let rewrites = IncrementalBTree::new(&all_entries, &old_leaves, &dirty).plan(|_, _| true);
        let mut rebuilt_leaves = Vec::new();
        for rewrite in rewrites {
            match rewrite {
                LeafRewrite::Reuse(leaf) => rebuilt_leaves.push(leaf),
                LeafRewrite::Write(entries) => {
                    for chunk in form_leaf_groups(&entries)? {
                        let (offset, object_id) = self.write_form_leaf(chunk)?;
                        rebuilt_leaves.push(FormLeaf {
                            offset,
                            object_id,
                            entries: chunk.to_vec(),
                        });
                    }
                }
            }
        }
        rebuilt_leaves.sort_by(|left, right| leaf_first_key(left).cmp(leaf_first_key(right)));
        let root_node = if form_leaf_directory_is_compatible(&old_leaves, &rebuilt_leaves) {
            let old_root = self.forms.tree.root.take().ok_or(Error::CorruptRecord)?;
            self.rewrite_compatible_form_tree(old_root, &rebuilt_leaves)?
        } else {
            self.write_form_tree_for_leaves(&rebuilt_leaves)?
        };
        let root = root_node.offset();
        self.forms.tree.replace_topology(root_node, rebuilt_leaves);
        Ok(root)
    }

    pub(super) fn latest_form_definitions(&self) -> Result<Vec<FormDefinition>> {
        self.ensure_forms_loaded()?;
        let mut latest = BTreeMap::<FormTypeId, FormDefinition>::new();
        for definition in self
            .forms
            .definitions
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .values()
        {
            let replace = latest
                .get(&definition.type_id)
                .is_none_or(|existing| definition.revision > existing.revision);
            if replace {
                latest.insert(definition.type_id.clone(), definition.clone());
            }
        }
        Ok(latest.into_values().collect())
    }

    pub(super) fn latest_form_definition_by_type(
        &self,
        type_id: &FormTypeId,
    ) -> Result<Option<FormDefinition>> {
        self.ensure_forms_loaded()?;
        Ok(self
            .forms
            .definitions
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .values()
            .filter(|definition| &definition.type_id == type_id)
            .max_by_key(|definition| definition.revision)
            .cloned())
    }

    pub(super) fn ensure_forms_loaded(&self) -> Result<()> {
        if !self.forms.is_loaded() {
            let (definitions, records) = if self.forms.tree.root_offset == 0 {
                (BTreeMap::new(), BTreeMap::new())
            } else {
                let (definitions, records, _, _) =
                    self.decode_form_btree(self.forms.tree.root_offset)?;
                (definitions, records)
            };
            self.forms.install(definitions, records);
        }
        Ok(())
    }

    fn decode_form_btree(&self, root_offset: u64) -> Result<FormTreeDecodeResult> {
        let mut definitions = BTreeMap::new();
        let mut records = BTreeMap::new();
        let root = self.decode_form_node_into(root_offset, &mut definitions, &mut records, 0)?;
        let mut leaves = Vec::new();
        root.collect_leaves(&mut leaves);
        leaves.sort_by(|left, right| {
            let left_key = left
                .entries
                .first()
                .map(|entry| entry.key.as_str())
                .unwrap_or("");
            let right_key = right
                .entries
                .first()
                .map(|entry| entry.key.as_str())
                .unwrap_or("");
            left_key.cmp(right_key)
        });
        Ok((definitions, records, root, leaves))
    }

    fn decode_form_node_into(
        &self,
        offset: u64,
        definitions: &mut BTreeMap<String, FormDefinition>,
        records: &mut BTreeMap<LockboxPath, FormRecord>,
        depth: usize,
    ) -> Result<FormTreeNode> {
        if depth > 8 {
            return Err(Error::CorruptRecord);
        }
        let (node, object_id) = self.read_form_node(offset)?;
        match node {
            FormNode::Leaf(entries) => {
                let leaf_entries = entries.clone();
                for entry in entries {
                    match entry.value {
                        FormEntryValue::Definition(definition) => {
                            definitions.insert(entry.key, definition);
                        }
                        FormEntryValue::Record(record) => {
                            records.insert(record.path.clone(), record);
                        }
                    }
                }
                Ok(FormTreeNode::Leaf(FormLeaf {
                    offset,
                    object_id,
                    entries: leaf_entries,
                }))
            }
            FormNode::Internal(children) => {
                let mut nodes = Vec::with_capacity(children.len());
                for child in children {
                    nodes.push(self.decode_form_node_into(
                        child.offset,
                        definitions,
                        records,
                        depth + 1,
                    )?);
                }
                Ok(FormTreeNode::Internal(FormInternal {
                    offset,
                    object_id,
                    children: nodes,
                }))
            }
        }
    }

    fn read_form_node(&self, offset: u64) -> Result<(FormNode, u64)> {
        let form_object = self.read_form_object_secure(offset)?;
        if !matches!(
            form_object.kind,
            PageObjectKind::FormLeaf | PageObjectKind::FormInternal
        ) {
            return Err(Error::CorruptRecord);
        }
        let payload = form_object.secure_payload().ok_or(Error::CorruptRecord)?;
        Ok((decode_form_node_secure(payload)?, form_object.id))
    }

    fn read_form_object_secure(&self, offset: u64) -> Result<PageObject> {
        self.with_secure_page(offset, |page| {
            if page.objects.len() != 1 {
                return Err(Error::CorruptRecord);
            }
            Ok(page.objects[0].clone())
        })
    }

    fn collect_form_tree_redactions(
        &self,
        offset: u64,
        depth: usize,
        redactions: &mut Vec<(u64, u64)>,
    ) -> Result<()> {
        if depth > 8 {
            return Err(Error::CorruptRecord);
        }
        let form_object = self.read_form_object_secure(offset)?;
        if !matches!(
            form_object.kind,
            PageObjectKind::FormLeaf | PageObjectKind::FormInternal
        ) {
            return Err(Error::CorruptRecord);
        }
        redactions.push((offset, form_object.id));
        if form_object.kind == PageObjectKind::FormInternal {
            let payload = form_object.secure_payload().ok_or(Error::CorruptRecord)?;
            let FormNode::Internal(children) = decode_form_node_secure(payload)? else {
                return Err(Error::CorruptRecord);
            };
            for child in children {
                self.collect_form_tree_redactions(child.offset, depth + 1, redactions)?;
            }
        }
        Ok(())
    }

    fn write_form_tree_for_leaves(&mut self, leaves: &[FormLeaf]) -> Result<FormTreeNode> {
        if leaves.len() == 1 {
            return Ok(FormTreeNode::Leaf(leaves[0].clone()));
        }
        let mut level = leaves
            .iter()
            .cloned()
            .map(FormTreeNode::Leaf)
            .collect::<Vec<_>>();
        while level.len() > 1 {
            let mut next_level = Vec::new();
            let mut child_cursor = 0usize;
            let children = level
                .iter()
                .map(|node| FormChild {
                    first_key: node.first_key().to_string(),
                    offset: node.offset(),
                })
                .collect::<Vec<_>>();
            for chunk in form_child_groups(&children)? {
                let (offset, object_id) = self.write_form_internal(chunk)?;
                let start = child_cursor;
                let end = start + chunk.len();
                child_cursor = end;
                let child_nodes = level[start..end].to_vec();
                next_level.push(FormTreeNode::Internal(FormInternal {
                    offset,
                    object_id,
                    children: child_nodes,
                }));
            }
            level = next_level;
        }
        Ok(level.remove(0))
    }

    fn rewrite_compatible_form_tree(
        &mut self,
        node: FormTreeNode,
        new_leaves: &[FormLeaf],
    ) -> Result<FormTreeNode> {
        match node {
            FormTreeNode::Leaf(old_leaf) => {
                let Some(new_leaf) = new_leaves
                    .iter()
                    .find(|leaf| leaf_first_key(leaf) == leaf_first_key(&old_leaf))
                    .cloned()
                else {
                    return Err(Error::CorruptRecord);
                };
                Ok(FormTreeNode::Leaf(new_leaf))
            }
            FormTreeNode::Internal(old_internal) => {
                let old_offset = old_internal.offset;
                let old_object_id = old_internal.object_id;
                let mut changed = false;
                let mut children = Vec::with_capacity(old_internal.children.len());
                for child in old_internal.children {
                    let child_offset = child.offset();
                    let child_first_key = child.first_key().to_string();
                    let rewritten = self.rewrite_compatible_form_tree(child, new_leaves)?;
                    if rewritten.offset() != child_offset
                        || rewritten.first_key() != child_first_key.as_str()
                    {
                        changed = true;
                    }
                    children.push(rewritten);
                }
                if !changed {
                    return Ok(FormTreeNode::Internal(FormInternal {
                        offset: old_offset,
                        object_id: old_object_id,
                        children,
                    }));
                }
                let form_children = children
                    .iter()
                    .map(|child| FormChild {
                        first_key: child.first_key().to_string(),
                        offset: child.offset(),
                    })
                    .collect::<Vec<_>>();
                let (offset, object_id) = self.write_form_internal(&form_children)?;
                Ok(FormTreeNode::Internal(FormInternal {
                    offset,
                    object_id,
                    children,
                }))
            }
        }
    }

    fn write_form_leaf(&mut self, entries: &[crate::form_btree::FormEntry]) -> Result<(u64, u64)> {
        let payload = encode_form_leaf_secure(entries)?;
        self.sequence += 1;
        let object_id = self.sequence;
        Ok((
            self.append_form_page_secure(PageObjectKind::FormLeaf, payload)?,
            object_id,
        ))
    }

    fn write_form_internal(&mut self, children: &[FormChild]) -> Result<(u64, u64)> {
        let payload = SecureVec::try_from_vec(encode_form_internal(children)?)?;
        self.sequence += 1;
        let object_id = self.sequence;
        Ok((
            self.append_form_page_secure(PageObjectKind::FormInternal, payload)?,
            object_id,
        ))
    }

    fn append_form_page_secure(
        &mut self,
        kind: PageObjectKind,
        mut payload: SecureVec,
    ) -> Result<u64> {
        self.flush_dirty_pages()?;
        let mut content_key = self.key.with_bytes(derive_page_content_key)?;
        let sequence = self.staged.sequence;
        let page_offset = self
            .page_manager
            .borrow_mut()
            .append_secure_single_object_page(
                &mut self.storage,
                SecurePageAppend {
                    lockbox_id: self.lockbox_id,
                    content_key: &content_key,
                    sequence,
                    kind,
                    object_id: sequence,
                    payload: &payload,
                },
            )?;
        content_key.zeroize();
        payload.zeroize()?;
        Ok(page_offset)
    }
}

fn leaf_first_key(leaf: &FormLeaf) -> &str {
    leaf.entries
        .first()
        .map(|entry| entry.key.as_str())
        .unwrap_or("")
}

fn form_leaf_directory_is_compatible(old: &[FormLeaf], new: &[FormLeaf]) -> bool {
    old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| leaf_first_key(old) == leaf_first_key(new))
}

fn form_entries_overlap_dirty(entries: &[FormEntry], dirty_keys: &BTreeSet<String>) -> bool {
    let Some(first) = entries.first().map(|entry| entry.key.as_str()) else {
        return false;
    };
    let last = entries
        .last()
        .map(|entry| entry.key.as_str())
        .unwrap_or(first);
    dirty_keys
        .iter()
        .any(|key| key.as_str() >= first && key.as_str() <= last)
}

fn collect_dirty_form_tree_redactions_from_node(
    node: &FormTreeNode,
    dirty_keys: &BTreeSet<String>,
    redactions: &mut Vec<(u64, u64)>,
) -> bool {
    match node {
        FormTreeNode::Leaf(leaf) => {
            let overlaps = form_entries_overlap_dirty(&leaf.entries, dirty_keys);
            if overlaps {
                redactions.push((leaf.offset, leaf.object_id));
            }
            overlaps
        }
        FormTreeNode::Internal(internal) => {
            let mut changed = false;
            for child in &internal.children {
                changed |=
                    collect_dirty_form_tree_redactions_from_node(child, dirty_keys, redactions);
            }
            if changed {
                redactions.push((internal.offset, internal.object_id));
            }
            changed
        }
    }
}
