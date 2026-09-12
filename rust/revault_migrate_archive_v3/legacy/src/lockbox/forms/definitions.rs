use super::*;

impl<State> Lockbox<State> {
    /// Creates a form definition, or creates a new revision of its alias.
    ///
    /// New definitions receive a random stable type id and an empty description.
    pub fn define_form(
        &mut self,
        alias: &str,
        name: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        self.define_form_with_description(alias, name, "", fields)
    }

    /// Creates or revises a form definition with descriptive text.
    ///
    /// Aliases are convenient names but may become ambiguous after imported
    /// definitions; use a type id when deterministic resolution is required.
    pub fn define_form_with_description(
        &mut self,
        alias: &str,
        name: &str,
        description: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        let alias = FormDefinition::validated_alias(alias)?;
        match self.resolve_form_definition(&alias) {
            Ok(existing) => {
                self.revise_form_definition(&existing.type_id, name, description, fields)
            }
            Err(Error::NotFound(_)) => {
                let type_id = FormTypeId::new_random()?;
                self.define_form_with_type_id_and_description(
                    type_id,
                    &alias,
                    name,
                    description,
                    fields,
                )
            }
            Err(err) => Err(err),
        }
    }

    /// Creates or revises a definition using a caller-supplied stable type id.
    ///
    /// This is intended for synchronization and migration code that must
    /// preserve identity across lockboxes.
    pub fn define_form_with_type_id(
        &mut self,
        type_id: FormTypeId,
        alias: &str,
        name: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        self.define_form_with_type_id_and_description(type_id, alias, name, "", fields)
    }

    /// Creates or revises a fully identified and described form definition.
    pub fn define_form_with_type_id_and_description(
        &mut self,
        type_id: FormTypeId,
        alias: &str,
        name: &str,
        description: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        let alias = FormDefinition::validated_alias(alias)?;
        if self.latest_form_definition_by_type(&type_id)?.is_some() {
            return self.revise_form_definition(&type_id, name, description, fields);
        }
        let definition = FormDefinition::validated(type_id, alias, 1, name, description, fields)?;
        self.ensure_forms_loaded()?;
        self.forms.put_definition(definition.clone())?;
        Ok(definition)
    }

    /// Appends the next revision for an existing form type.
    ///
    /// Existing records keep their captured labels and revision until a field
    /// is next updated.
    pub fn revise_form_definition(
        &mut self,
        type_id: &FormTypeId,
        name: &str,
        description: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        let previous = self
            .latest_form_definition_by_type(type_id)?
            .ok_or_else(|| Error::NotFound(format!("form type {type_id}")))?;
        for previous_field in &previous.fields {
            if previous_field.kind.is_secret()
                && fields
                    .iter()
                    .any(|field| field.id == previous_field.id && !field.kind.is_secret())
            {
                return Err(Error::InvalidOperation(format!(
                    "form field {} is secret; remove it in one definition revision before recreating it as non-secret",
                    previous_field.id
                )));
            }
        }
        let definition = FormDefinition::validated(
            type_id.clone(),
            previous.alias.clone(),
            previous.revision + 1,
            name,
            description,
            fields,
        )?;
        self.forms.put_definition(definition.clone())?;
        Ok(definition)
    }

    /// Resolves the newest form definition by type id or unambiguous alias.
    pub fn resolve_form_definition(&self, reference: &str) -> Result<FormDefinition> {
        self.ensure_forms_loaded()?;
        if let Ok(type_id) = FormTypeId::new(reference) {
            return self
                .latest_form_definition_by_type(&type_id)?
                .ok_or_else(|| Error::NotFound(format!("form type {type_id}")));
        }
        let alias = FormDefinition::validated_alias(reference)?;
        let matches = self
            .latest_form_definitions()?
            .into_iter()
            .filter(|definition| definition.alias == alias)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [definition] => Ok(definition.clone()),
            [] => Err(Error::NotFound(format!("form alias {alias}"))),
            _ => Err(Error::InvalidOperation(format!(
                "form alias {alias} is ambiguous; use a form type id"
            ))),
        }
    }

    /// Lists the newest revision of every form type in stable order.
    pub fn list_form_definitions(&self) -> Result<Vec<FormDefinition>> {
        self.latest_form_definitions()
    }

    /// Lists every stored revision for `type_id`, oldest first.
    pub fn list_form_definition_revisions(
        &self,
        type_id: &FormTypeId,
    ) -> Result<Vec<FormDefinition>> {
        self.ensure_forms_loaded()?;
        let mut definitions = self
            .forms
            .definitions
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .values()
            .filter(|definition| definition.type_id == *type_id)
            .cloned()
            .collect::<Vec<_>>();
        definitions.sort_by_key(|definition| definition.revision);
        Ok(definitions)
    }

    /// Imports an exact definition revision without renumbering it.
    ///
    /// Re-importing identical content is idempotent; conflicting content for
    /// the same type id and revision is rejected.
    pub fn import_form_definition(&mut self, definition: FormDefinition) -> Result<FormDefinition>
    where
        State: crate::WritableLockboxState,
    {
        if definition.revision == 0 {
            return Err(Error::InvalidInput(
                "form definition revision must be greater than zero".to_string(),
            ));
        }
        let definition = FormDefinition::validated(
            definition.type_id,
            definition.alias,
            definition.revision,
            &definition.name,
            &definition.description,
            definition.fields,
        )?;
        self.ensure_forms_loaded()?;
        let key = definition_key(&definition.type_id, definition.revision);
        let mut definitions = self.staged.forms.definitions.borrow_mut();
        let definitions = definitions.as_mut().ok_or(Error::CorruptRecord)?;
        if let Some(existing) = definitions.get(&key) {
            if existing == &definition {
                return Ok(existing.clone());
            }
            return Err(Error::InvalidOperation(format!(
                "form definition {} revision {} already exists with different content",
                definition.type_id, definition.revision
            )));
        }
        definitions.insert(key.clone(), definition.clone());
        self.staged.forms.tree.dirty_keys.insert(key);
        self.staged.forms.dirty = true;
        Ok(definition)
    }
}
