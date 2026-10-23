//! Optional descriptions use the existing encrypted variable representation.
//! Keeping the wire representation unchanged lets format-4 readers retain them.
use super::Lockbox;
use crate::{Error, FormDefinition, LockboxPath, Result, VariableName, WritableLockboxState};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(super) fn metadata_name(kind: &str, identity: &str) -> Result<VariableName> {
    let digest = Sha256::digest(identity.as_bytes());
    let key = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    VariableName::new(format!("/.revault/descriptions/{kind}/d_{key}"))
}

impl<State> Lockbox<State> {
    /// Return field guidance for an exact definition revision. This accompanies
    /// a definition when copying it between vaults or lockboxes.
    pub fn form_field_descriptions(
        &self,
        definition: &FormDefinition,
    ) -> Result<BTreeMap<String, String>> {
        let identity = format!("{}/{}", definition.type_id, definition.revision);
        match self.get_variable(&metadata_name("fields", &identity)?)? {
            None => Ok(BTreeMap::new()),
            Some(value) => {
                let descriptions: BTreeMap<String, String> =
                    serde_json::from_str(&value).map_err(|_| Error::CorruptRecord)?;
                for (field, text) in &descriptions {
                    Self::require_description_field(definition, field)
                        .map_err(|_| Error::CorruptRecord)?;
                    crate::security::validate_variable_value_ref(text)
                        .map_err(|_| Error::CorruptRecord)?;
                }
                Ok(descriptions)
            }
        }
    }

    /// Return guidance for a field in the latest matching definition.
    pub fn form_field_description(&self, reference: &str, field: &str) -> Result<Option<String>> {
        let definition = self.resolve_form_definition(reference)?;
        Self::require_description_field(&definition, field)?;
        Ok(self.form_field_descriptions(&definition)?.remove(field))
    }

    fn require_description_field(definition: &FormDefinition, field: &str) -> Result<()> {
        if definition.fields.iter().any(|item| item.id == field) {
            Ok(())
        } else {
            Err(Error::NotFound(format!("form field {field}")))
        }
    }

    pub(super) fn store_form_field_descriptions(
        &mut self,
        definition: &FormDefinition,
        descriptions: &BTreeMap<String, String>,
    ) -> Result<()>
    where
        State: WritableLockboxState,
    {
        for (field, text) in descriptions {
            Self::require_description_field(definition, field)?;
            crate::security::validate_variable_value_ref(text)?;
        }
        let identity = format!("{}/{}", definition.type_id, definition.revision);
        let key = metadata_name("fields", &identity)?;
        if descriptions.is_empty() {
            self.delete_variable(&key)
        } else {
            let encoded = serde_json::to_string(descriptions).map_err(|_| Error::CorruptRecord)?;
            self.set_variable(&key, &encoded)
        }
    }

    /// Import an exact definition revision together with its field guidance.
    /// Existing revisions may only be reimported with identical guidance.
    pub fn import_form_definition_with_descriptions(
        &mut self,
        definition: FormDefinition,
        descriptions: BTreeMap<String, String>,
    ) -> Result<FormDefinition>
    where
        State: WritableLockboxState,
    {
        for (field, text) in &descriptions {
            Self::require_description_field(&definition, field)?;
            crate::security::validate_variable_value_ref(text)?;
        }
        let encoded = serde_json::to_string(&descriptions).map_err(|_| Error::CorruptRecord)?;
        crate::security::validate_variable_value_ref(&encoded)?;
        let exists = self
            .list_form_definition_revisions(&definition.type_id)?
            .iter()
            .any(|item| item.revision == definition.revision);
        if exists && self.form_field_descriptions(&definition)? != descriptions {
            return Err(Error::InvalidOperation(
                "definition revision already exists with different field descriptions".to_string(),
            ));
        }
        let definition = self.import_form_definition(definition)?;
        self.store_form_field_descriptions(&definition, &descriptions)?;
        Ok(definition)
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

    /// Create a definition revision with updated guidance for one field.
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
        let definition = self.resolve_form_definition(reference)?;
        Self::require_description_field(&definition, field)?;
        let mut descriptions = self.form_field_descriptions(&definition)?;
        if descriptions.get(field).map(String::as_str) == text {
            return Ok(definition);
        }
        if let Some(text) = text {
            descriptions.insert(field.to_string(), text.to_string());
        } else {
            descriptions.remove(field);
        }
        let encoded = serde_json::to_string(&descriptions).map_err(|_| Error::CorruptRecord)?;
        crate::security::validate_variable_value_ref(&encoded)?;
        let revised = self.revise_form_definition(
            &definition.type_id,
            &definition.name,
            &definition.description,
            definition.fields,
        )?;
        self.store_form_field_descriptions(&revised, &descriptions)?;
        Ok(revised)
    }

    /// Read explanatory text for an existing variable without reading its value.
    pub fn variable_description(&self, name: &VariableName) -> Result<Option<String>> {
        if self.variable_sensitivity(name)?.is_none() {
            return Err(Error::NotFound(format!("variable {name}")));
        }
        self.get_variable(&metadata_name("variable", name.as_str())?)
    }

    /// Set multiline UTF-8 guidance independently of a variable's sensitivity.
    /// Commit the lockbox to persist the change. Empty text is distinct from absence.
    pub fn set_variable_description(&mut self, name: &VariableName, text: &str) -> Result<()>
    where
        State: WritableLockboxState,
    {
        self.variable_description(name)?;
        self.set_variable(&metadata_name("variable", name.as_str())?, text)
    }

    /// Clear a variable's description; the variable itself must exist.
    pub fn clear_variable_description(&mut self, name: &VariableName) -> Result<()>
    where
        State: WritableLockboxState,
    {
        self.variable_description(name)?;
        self.delete_variable(&metadata_name("variable", name.as_str())?)
    }

    /// Read the description of an individual stored form record.
    pub fn form_record_description(&self, path: &LockboxPath) -> Result<Option<String>> {
        let path = path.file_path()?;
        if self.get_form_record(&path)?.is_none() {
            return Err(Error::NotFound(format!("form record {path}")));
        }
        self.get_variable(&metadata_name("record", path.as_str())?)
    }

    /// Set a form record's own description without changing its field values.
    pub fn set_form_record_description(&mut self, path: &LockboxPath, text: &str) -> Result<()>
    where
        State: WritableLockboxState,
    {
        let path = path.file_path()?;
        self.form_record_description(&path)?;
        self.set_variable(&metadata_name("record", path.as_str())?, text)
    }

    /// Clear a record's description; the record itself must exist.
    pub fn clear_form_record_description(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: WritableLockboxState,
    {
        let path = path.file_path()?;
        self.form_record_description(&path)?;
        self.delete_variable(&metadata_name("record", path.as_str())?)
    }
}
