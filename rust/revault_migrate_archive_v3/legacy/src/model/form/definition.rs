use std::collections::BTreeSet;

use super::field::FormFieldKind;
use super::validation::{validate_identifier, validate_text};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Stable UUID-shaped identifier shared by all revisions of a form definition.
pub struct FormTypeId(String);

impl FormTypeId {
    /// Parses and normalizes a UUID-shaped hexadecimal form type id.
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        if value.len() != 36 || !value.chars().all(|ch| ch == '-' || ch.is_ascii_hexdigit()) {
            return Err(Error::InvalidInput(format!(
                "invalid form type id: {value}"
            )));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }

    pub(crate) fn new_random() -> Result<Self> {
        Ok(Self(crate::LockboxId::new_random()?.to_string()))
    }

    /// Returns the normalized lowercase identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FormTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Versioned schema describing the fields accepted by a form record.
pub struct FormDefinition {
    /// Stable identity shared by all revisions.
    pub type_id: FormTypeId,
    /// Human-friendly, case-sensitive lookup name.
    pub alias: String,
    /// Monotonically increasing schema revision, beginning at one.
    pub revision: u32,
    /// Display name for the form type.
    pub name: String,
    /// Optional explanatory text shown to users.
    pub description: String,
    /// Ordered field definitions captured by this revision.
    pub fields: Vec<FormFieldDefinition>,
}

impl FormDefinition {
    pub(crate) fn validated_alias(value: &str) -> Result<String> {
        validate_identifier(value, "form alias")
    }

    pub(crate) fn validated_name(value: &str) -> Result<String> {
        validate_text(value, "form name")?;
        Ok(value.to_string())
    }

    pub(crate) fn validated_description(value: &str) -> Result<String> {
        validate_text(value, "form description")?;
        Ok(value.to_string())
    }

    pub(crate) fn validated(
        type_id: FormTypeId,
        alias: String,
        revision: u32,
        name: &str,
        description: &str,
        fields: Vec<FormFieldDefinition>,
    ) -> Result<Self> {
        if fields.is_empty() {
            return Err(Error::InvalidInput(
                "form definition requires at least one field".to_string(),
            ));
        }
        let mut seen = BTreeSet::new();
        let mut validated_fields = Vec::with_capacity(fields.len());
        for field in fields {
            let id = FormFieldDefinition::validated_id(&field.id)?;
            if !seen.insert(id.clone()) {
                return Err(Error::InvalidInput(format!(
                    "duplicate form field id: {id}"
                )));
            }
            validated_fields.push(FormFieldDefinition {
                id,
                label: FormFieldDefinition::validated_label(&field.label)?,
                kind: field.kind,
                required: field.required,
            });
        }
        Ok(Self {
            type_id,
            alias: Self::validated_alias(&alias)?,
            revision,
            name: Self::validated_name(name)?,
            description: Self::validated_description(description)?,
            fields: validated_fields,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Schema for one named field in a form definition.
pub struct FormFieldDefinition {
    /// Stable, case-sensitive machine-readable identifier within the form.
    pub id: String,
    /// User-facing field label.
    pub label: String,
    /// Validation and sensitivity category.
    pub kind: FormFieldKind,
    /// Whether callers should require a value before considering a form complete.
    pub required: bool,
}

impl FormFieldDefinition {
    pub(crate) fn validated_id(value: &str) -> Result<String> {
        validate_identifier(value, "form field id")
    }

    pub(crate) fn validated_label(value: &str) -> Result<String> {
        validate_text(value, "form field label")?;
        Ok(value.to_string())
    }
}
