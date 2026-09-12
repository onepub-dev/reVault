use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::form_btree::{definition_key, record_key, FormLeaf, FormTreeNode};
use crate::incremental_btree::PersistedBTree;
use crate::{Error, FormDefinition, FormRecord, LockboxPath, Result};

/// In-memory form catalog, records, and persisted tree state.
#[derive(Debug, Clone)]
pub(super) struct FormStore {
    pub(super) definitions: RefCell<Option<BTreeMap<String, FormDefinition>>>,
    pub(super) records: RefCell<Option<BTreeMap<LockboxPath, FormRecord>>>,
    pub(super) tree: PersistedBTree<FormTreeNode, FormLeaf, String>,
    pub(super) dirty: bool,
}

impl FormStore {
    pub(super) fn loaded() -> Self {
        Self {
            definitions: RefCell::new(Some(BTreeMap::new())),
            records: RefCell::new(Some(BTreeMap::new())),
            tree: PersistedBTree::default(),
            dirty: false,
        }
    }

    pub(super) fn unloaded() -> Self {
        Self {
            definitions: RefCell::new(None),
            records: RefCell::new(None),
            tree: PersistedBTree::default(),
            dirty: false,
        }
    }

    pub(super) fn is_loaded(&self) -> bool {
        self.definitions.borrow().is_some() && self.records.borrow().is_some()
    }

    pub(super) fn install(
        &self,
        definitions: BTreeMap<String, FormDefinition>,
        records: BTreeMap<LockboxPath, FormRecord>,
    ) {
        self.definitions.replace(Some(definitions));
        self.records.replace(Some(records));
    }

    pub(super) fn put_definition(&mut self, definition: FormDefinition) -> Result<()> {
        let key = definition_key(&definition.type_id, definition.revision);
        self.definitions
            .get_mut()
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .insert(key.clone(), definition);
        self.tree.dirty_keys.insert(key);
        self.dirty = true;
        Ok(())
    }

    pub(super) fn put_definition_at(
        &mut self,
        key: String,
        definition: FormDefinition,
    ) -> Result<()> {
        self.definitions
            .get_mut()
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .insert(key.clone(), definition);
        self.tree.dirty_keys.insert(key);
        self.dirty = true;
        Ok(())
    }

    pub(super) fn put_record(&mut self, path: LockboxPath, record: FormRecord) -> Result<()> {
        self.records
            .get_mut()
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .insert(path.clone(), record);
        self.tree.dirty_keys.insert(record_key(&path));
        self.dirty = true;
        Ok(())
    }
}
