use super::*;

impl<State> Lockbox<State> {
    /// Sets a non-secret field after validating it against the latest definition.
    pub fn set_form_field_normal(
        &mut self,
        path: &LockboxPath,
        field_id: &str,
        value: &str,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.set_form_field(path, field_id, FormValue::Normal(value.to_string()))
    }

    /// Copies a secret into the named secret field.
    ///
    /// The supplied [`SecretString`] remains owned by the caller; the lockbox
    /// stores an independent secure clone. If the latest definition currently
    /// declares the field as non-secret, this operation appends a secret-field
    /// definition revision and securely upgrades that field in every record of
    /// the same form type. A secret field cannot be downgraded by setting a
    /// normal value.
    pub fn set_form_field_secret(
        &mut self,
        path: &LockboxPath,
        field_id: &str,
        value: &SecretString,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        self.set_form_field(
            path,
            field_id,
            FormValue::Secret(std::sync::Arc::new(value.try_clone()?)),
        )
    }

    /// Sets a normal or secret field and captures its current label and revision.
    ///
    /// Prefer [`Lockbox::set_form_field_normal`] and
    /// [`Lockbox::set_form_field_secret`] when the sensitivity is known.
    pub fn set_form_field(
        &mut self,
        path: &LockboxPath,
        field_id: &str,
        value: FormValue,
    ) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let field_id = FormFieldDefinition::validated_id(field_id)?;
        self.ensure_forms_loaded()?;
        let type_id = self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .get(path)
            .ok_or_else(|| Error::NotFound(format!("form record {path}")))?
            .type_id
            .clone();
        let mut definition = self
            .latest_form_definition_by_type(&type_id)?
            .ok_or_else(|| Error::NotFound(format!("form type {type_id}")))?;
        let mut field = definition
            .fields
            .iter()
            .find(|field| field.id == field_id)
            .cloned()
            .ok_or_else(|| Error::InvalidInput(format!("unknown form field: {field_id}")))?;
        if value.is_secret() && !field.kind.is_secret() {
            definition = self.upgrade_form_field_to_secret(&type_id, &field_id)?;
            field = definition
                .fields
                .iter()
                .find(|field| field.id == field_id)
                .cloned()
                .ok_or(Error::CorruptRecord)?;
        }
        field.kind.validate_value(&value)?;
        let value_record = FormFieldValue {
            field_id,
            captured_label: field.label.clone(),
            kind: field.kind,
            value,
        };
        let mut records = self.staged.forms.records.borrow_mut();
        let records = records.as_mut().ok_or(Error::CorruptRecord)?;
        let record = records
            .get_mut(path)
            .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
        match record
            .values
            .iter_mut()
            .find(|existing| existing.field_id == value_record.field_id)
        {
            Some(existing) => *existing = value_record,
            None => record.values.push(value_record),
        }
        record.definition_revision = definition.revision;
        record.definition_alias = definition.alias;
        self.staged.forms.tree.dirty_keys.insert(record_key(path));
        self.staged.forms.dirty = true;
        Ok(())
    }

    fn upgrade_form_field_to_secret(
        &mut self,
        type_id: &FormTypeId,
        field_id: &str,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        let previous = self
            .latest_form_definition_by_type(type_id)?
            .ok_or_else(|| Error::NotFound(format!("form type {type_id}")))?;
        let mut fields = previous.fields.clone();
        let field = fields
            .iter_mut()
            .find(|field| field.id == field_id)
            .ok_or_else(|| Error::InvalidInput(format!("unknown form field: {field_id}")))?;
        if field.kind.is_secret() {
            return Ok(previous);
        }
        field.kind = FormFieldKind::Secret;
        let definition = FormDefinition::validated(
            previous.type_id.clone(),
            previous.alias.clone(),
            previous.revision + 1,
            &previous.name,
            &previous.description,
            fields,
        )?;
        let upgraded_label = definition
            .fields
            .iter()
            .find(|field| field.id == field_id)
            .ok_or(Error::CorruptRecord)?
            .label
            .clone();

        let converted = self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .iter()
            .filter(|(_, record)| record.type_id == *type_id)
            .filter_map(|(path, record)| {
                record
                    .values
                    .iter()
                    .find(|value| value.field_id == field_id)
                    .and_then(|value| match &value.value {
                        FormValue::Normal(value) => Some(
                            SecretString::try_from_slice(value.as_bytes())
                                .map_err(Error::from)
                                .map(|value| (path.clone(), Arc::new(value))),
                        ),
                        FormValue::Secret(_) => None,
                    })
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let definition_key = definition_key(&definition.type_id, definition.revision);
        self.forms
            .definitions
            .borrow_mut()
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .insert(definition_key.clone(), definition.clone());
        self.forms.tree.dirty_keys.insert(definition_key);

        let mut records = self.staged.forms.records.borrow_mut();
        let records = records.as_mut().ok_or(Error::CorruptRecord)?;
        for record in records
            .values_mut()
            .filter(|record| record.type_id == *type_id)
        {
            if let Some(value) = record
                .values
                .iter_mut()
                .find(|value| value.field_id == field_id)
            {
                if let Some(secret) = converted.get(&record.path) {
                    if let FormValue::Normal(plaintext) = &mut value.value {
                        plaintext.zeroize();
                    }
                    value.value = FormValue::Secret(Arc::clone(secret));
                }
                value.kind = FormFieldKind::Secret;
                value.captured_label = upgraded_label.clone();
            }
            record.definition_alias = definition.alias.clone();
            record.definition_revision = definition.revision;
            self.staged
                .forms
                .tree
                .dirty_keys
                .insert(record_key(&record.path));
        }
        self.staged.forms.dirty = true;
        Ok(definition)
    }

    /// Returns a cloned field value, or `None` when the record or field is absent.
    pub fn get_form_field(
        &self,
        path: &LockboxPath,
        field_id: &str,
    ) -> Result<Option<FormFieldValue>> {
        let field_id = FormFieldDefinition::validated_id(field_id)?;
        self.ensure_forms_loaded()?;
        Ok(self
            .forms
            .records
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .get(path)
            .and_then(|record| {
                record
                    .values
                    .iter()
                    .find(|value| value.field_id == field_id)
                    .cloned()
            }))
    }
}
