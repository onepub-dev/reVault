//! Descriptions belong to the variable, field definition, or record they describe.
use super::Lockbox;
use crate::{Error, FormDefinition, LockboxPath, Result, VariableName, WritableLockboxState};
use std::collections::BTreeMap;

impl<State> Lockbox<State> {
    /// Return field guidance captured by an exact definition revision.
    pub fn form_field_descriptions(
        &self,
        definition: &FormDefinition,
    ) -> Result<BTreeMap<String, String>> {
        Ok(definition
            .fields
            .iter()
            .filter_map(|field| {
                field
                    .description
                    .clone()
                    .map(|text| (field.id.clone(), text))
            })
            .collect())
    }

    /// Return guidance for a field in the latest matching definition.
    pub fn form_field_description(&self, reference: &str, field: &str) -> Result<Option<String>> {
        let definition = self.resolve_form_definition(reference)?;
        definition
            .fields
            .iter()
            .find(|item| item.id == field)
            .map(|item| item.description.clone())
            .ok_or_else(|| Error::NotFound(format!("form field {field}")))
    }

    /// Import an exact definition revision with field descriptions. Descriptions
    /// are part of the definition and participate in revision conflict checks.
    pub fn import_form_definition_with_descriptions(
        &mut self,
        mut definition: FormDefinition,
        descriptions: BTreeMap<String, String>,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        for (field, text) in &descriptions {
            if !definition.fields.iter().any(|item| &item.id == field) {
                return Err(Error::NotFound(format!("form field {field}")));
            }
            crate::security::validate_variable_value_ref(text)?;
        }
        for field in &mut definition.fields {
            field.description = descriptions.get(&field.id).cloned();
        }
        self.import_form_definition(definition)
    }

    /// Revise form guidance without replacing its name, fields, or record values.
    pub fn set_form_definition_description(
        &mut self,
        reference: &str,
        text: &str,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        let definition = self.resolve_form_definition(reference)?;
        if definition.description == text {
            return Ok(definition);
        }
        self.revise_form_definition(
            &definition.type_id,
            &definition.name,
            text,
            definition.fields,
        )
    }

    /// Clear form guidance, preserving the existing empty-description convention.
    pub fn clear_form_definition_description(&mut self, reference: &str) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        self.set_form_definition_description(reference, "")
    }

    /// Create a definition revision with updated multiline guidance for one field.
    pub fn set_form_field_description(
        &mut self,
        reference: &str,
        field: &str,
        text: &str,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        crate::security::validate_variable_value_ref(text)?;
        self.change_form_field_description(reference, field, Some(text))
    }

    /// Clear field guidance. Clearing absent guidance does not create a revision.
    pub fn clear_form_field_description(
        &mut self,
        reference: &str,
        field: &str,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        self.change_form_field_description(reference, field, None)
    }

    fn change_form_field_description(
        &mut self,
        reference: &str,
        field: &str,
        text: Option<&str>,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        let mut definition = self.resolve_form_definition(reference)?;
        let item = definition
            .fields
            .iter_mut()
            .find(|item| item.id == field)
            .ok_or_else(|| Error::NotFound(format!("form field {field}")))?;
        if item.description.as_deref() == text {
            return Ok(definition);
        }
        item.description = text.map(str::to_owned);
        self.revise_form_definition(
            &definition.type_id,
            &definition.name,
            &definition.description,
            definition.fields,
        )
    }

    /// Read explanatory text for an existing variable without reading its value.
    pub fn variable_description(&self, name: &VariableName) -> Result<Option<String>> {
        self.ensure_variables_loaded()?;
        self.variables
            .borrow()
            .as_ref()
            .ok_or(Error::CorruptRecord)?
            .get(name)
            .map(|value| value.description().clone())
            .ok_or_else(|| Error::NotFound(format!("variable {name}")))
    }

    /// Set multiline guidance independently of the variable's sensitivity.
    /// Commit to persist the change. Empty text differs from absence.
    pub fn set_variable_description(&mut self, name: &VariableName, text: &str) -> Result<()>
    where
        State: WritableLockboxState,
    {
        crate::security::validate_variable_value_ref(text)?;
        self.change_variable_description(name, Some(text.to_owned()))
    }

    /// Clear a variable's description; the variable itself must exist.
    pub fn clear_variable_description(&mut self, name: &VariableName) -> Result<()>
    where
        State: WritableLockboxState,
    {
        self.change_variable_description(name, None)
    }

    fn change_variable_description(
        &mut self,
        name: &VariableName,
        text: Option<String>,
    ) -> Result<()>
    where
        State: WritableLockboxState,
    {
        self.ensure_variables_loaded()?;
        let mut variables = self.staged.variables.borrow_mut();
        let value = variables
            .as_mut()
            .ok_or(Error::CorruptRecord)?
            .get_mut(name)
            .ok_or_else(|| Error::NotFound(format!("variable {name}")))?;
        if value.description() != &text {
            value.set_description(text);
            self.staged.dirty_variables = true;
        }
        Ok(())
    }

    /// Read the description of an individual stored form record.
    pub fn form_record_description(&self, path: &LockboxPath) -> Result<Option<String>> {
        self.get_form_record(&path.file_path()?)?
            .map(|record| record.description)
            .ok_or_else(|| Error::NotFound(format!("form record {path}")))
    }

    /// Set a record's own description without changing its field values.
    pub fn set_form_record_description(&mut self, path: &LockboxPath, text: &str) -> Result<()>
    where
        State: WritableLockboxState,
    {
        crate::security::validate_variable_value_ref(text)?;
        self.change_record_description(path, Some(text.to_owned()))
    }

    /// Clear a record's description; the record itself must exist.
    pub fn clear_form_record_description(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: WritableLockboxState,
    {
        self.change_record_description(path, None)
    }

    fn change_record_description(&mut self, path: &LockboxPath, text: Option<String>) -> Result<()>
    where
        State: WritableLockboxState,
    {
        let path = path.file_path()?;
        let mut record = self
            .get_form_record(&path)?
            .ok_or_else(|| Error::NotFound(format!("form record {path}")))?;
        if record.description == text {
            return Ok(());
        }
        record.description = text;
        self.forms.put_record(path, record)
    }
}
