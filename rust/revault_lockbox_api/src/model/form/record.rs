use super::definition::FormTypeId;
use super::field::FormFieldValue;
use super::validation::validate_text;
use crate::{Error, LockboxPath, Result};

#[derive(Debug, Clone)]
/// Stored form instance tied to a definition revision.
pub struct FormRecord {
    /// Logical lockbox path identifying this record.
    pub path: LockboxPath,
    /// User-facing record name.
    pub name: String,
    /// Stable form type identity.
    pub type_id: FormTypeId,
    /// Alias captured from the definition revision.
    pub definition_alias: String,
    /// Definition revision last applied to this record.
    pub definition_revision: u32,
    /// Values that have been assigned, in record order.
    pub values: Vec<FormFieldValue>,
}

impl FormRecord {
    pub(crate) fn validated_name(value: &str) -> Result<String> {
        validate_text(value, "form record name")?;
        if value.trim().is_empty() {
            return Err(Error::InvalidInput(
                "form record name cannot be empty".to_string(),
            ));
        }
        Ok(value.to_string())
    }
}
