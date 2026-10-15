use super::*;

impl<State> Lockbox<State> {
    /// Creates an empty record from the newest matching form definition.
    ///
    /// `type_reference` may be a type id or unambiguous alias. The record path
    /// must not already exist. Missing parent directories are created with the
    /// default directory permissions.
    pub fn create_form_record(
        &mut self,
        path: &LockboxPath,
        type_reference: &str,
        name: &str,
    ) -> Result<FormRecord>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let name = FormRecord::validated_name(name)?;
        self.ensure_forms_loaded()?;
        if self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .contains_key(&path)
        {
            return Err(Error::AlreadyExists(path.to_string()));
        }
        let definition = self.resolve_form_definition(type_reference)?;
        self.create_parent_dirs_for(&path)?;
        let record = FormRecord {
            path: path.clone(),
            name,
            type_id: definition.type_id,
            definition_alias: definition.alias,
            definition_revision: definition.revision,
            values: Vec::new(),
        };
        self.forms
            .records
            .borrow_mut()
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .insert(path, record.clone());
        self.forms.tree.dirty_keys.insert(record_key(&record.path));
        self.forms.dirty = true;
        Ok(record)
    }

    /// Restores an exact logical form record during archive migration.
    ///
    /// Unlike `create_form_record`, this preserves the definition revision and
    /// captured labels recorded by the source archive.
    #[doc(hidden)]
    #[cfg(feature = "migration")]
    pub fn import_migration_form_record(&mut self, mut record: FormRecord) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        record.path = record.path.file_path()?;
        record.name = FormRecord::validated_name(&record.name)?;
        self.ensure_forms_loaded()?;
        let definition = self
            .forms
            .definitions
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .get(&definition_key(&record.type_id, record.definition_revision))
            .cloned()
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "form type {} revision {}",
                    record.type_id, record.definition_revision
                ))
            })?;
        if definition.alias != record.definition_alias {
            return Err(Error::InvalidInput(
                "form record definition alias does not match its revision".to_string(),
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for value in &record.values {
            if !seen.insert(value.field_id.clone()) {
                return Err(Error::InvalidInput(format!(
                    "duplicate form field: {}",
                    value.field_id
                )));
            }
            let field = definition
                .fields
                .iter()
                .find(|field| field.id == value.field_id)
                .ok_or_else(|| {
                    Error::InvalidInput(format!("unknown form field: {}", value.field_id))
                })?;
            if field.kind != value.kind {
                return Err(Error::InvalidInput(format!(
                    "form field kind changed for {}",
                    value.field_id
                )));
            }
            value.kind.validate_value(&value.value)?;
        }
        let path = record.path.clone();
        if self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .contains_key(&path)
        {
            return Err(Error::AlreadyExists(path.to_string()));
        }
        self.create_parent_dirs_for(&path)?;
        self.set_form_record_value(path, record)
    }

    /// Returns a cloned form record, or `None` when `path` has no record.
    ///
    /// Secret field values remain in secure-memory containers.
    pub fn get_form_record(&self, path: &LockboxPath) -> Result<Option<FormRecord>> {
        self.ensure_forms_loaded()?;
        Ok(self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .get(path)
            .cloned())
    }

    /// Lists all form records in lockbox path order.
    pub fn list_form_records(&self) -> Result<Vec<FormRecord>> {
        self.ensure_forms_loaded()?;
        Ok(self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .values()
            .cloned()
            .collect())
    }

    /// Deletes the form record at `path` without deleting its definition.
    pub fn delete_form_record(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        self.ensure_forms_loaded()?;
        let mut records = self.staged.forms.records.borrow_mut();
        let records = records.as_mut().ok_or(Error::CorruptRecord)?;
        if records.remove(&path).is_none() {
            return Err(Error::NotFound(format!("form record {path}")));
        }
        self.staged.forms.tree.dirty_keys.insert(record_key(&path));
        self.staged.forms.dirty = true;
        Ok(())
    }

    /// Move one or more form records while preserving all field values.
    ///
    /// The complete move is validated before records are changed. Existing
    /// unrelated records are never overwritten.
    pub fn move_form_records(&mut self, moves: &[(LockboxPath, LockboxPath)]) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let moves = moves
            .iter()
            .map(|(source, destination)| Ok((source.file_path()?, destination.file_path()?)))
            .collect::<Result<Vec<_>>>()?;
        for (_, destination) in &moves {
            self.create_parent_dirs_for(destination)?;
        }
        self.ensure_forms_loaded()?;
        let mut records = self.staged.forms.records.borrow_mut();
        let records = records.as_mut().ok_or(Error::CorruptRecord)?;
        let sources = moves
            .iter()
            .map(|(source, _)| source.clone())
            .collect::<BTreeSet<_>>();
        if sources.len() != moves.len() {
            return Err(Error::InvalidInput(
                "a form record cannot be moved more than once".to_string(),
            ));
        }
        let mut destinations = BTreeSet::new();
        for (source, destination) in &moves {
            if !records.contains_key(source) {
                return Err(Error::NotFound(format!("form record {source}")));
            }
            if !destinations.insert(destination.clone()) {
                return Err(Error::AlreadyExists(destination.to_string()));
            }
            if source != destination
                && records.contains_key(destination)
                && !sources.contains(destination)
            {
                return Err(Error::AlreadyExists(destination.to_string()));
            }
        }
        let moved = moves
            .iter()
            .filter(|(source, destination)| source != destination)
            .map(|(source, destination)| {
                let mut record = records.remove(source).ok_or(Error::CorruptRecord)?;
                self.staged.forms.tree.dirty_keys.insert(record_key(source));
                record.path = destination.clone();
                self.staged
                    .forms
                    .tree
                    .dirty_keys
                    .insert(record_key(destination));
                Ok((destination.clone(), record))
            })
            .collect::<Result<Vec<_>>>()?;
        if !moved.is_empty() {
            records.extend(moved);
            self.staged.forms.dirty = true;
        }
        Ok(())
    }
}
