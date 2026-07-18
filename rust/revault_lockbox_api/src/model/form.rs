use std::sync::Arc;

use crate::{Error, LockboxPath, Result, SecretString};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormTypeId(String);

impl FormTypeId {
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
pub struct FormDefinition {
    pub type_id: FormTypeId,
    pub alias: String,
    pub revision: u32,
    pub name: String,
    pub description: String,
    pub fields: Vec<FormFieldDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldDefinition {
    pub id: String,
    pub label: String,
    pub kind: FormFieldKind,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldKind {
    Text,
    Secret,
    Url,
    Email,
    Date,
    Month,
    Notes,
    Number,
}

impl FormFieldKind {
    pub fn is_secret(self) -> bool {
        matches!(self, Self::Secret)
    }

    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Text => 1,
            Self::Secret => 2,
            Self::Url => 3,
            Self::Email => 4,
            Self::Date => 5,
            Self::Month => 6,
            Self::Notes => 7,
            Self::Number => 8,
        }
    }

    pub(crate) fn from_code(code: u8) -> Result<Self> {
        match code {
            1 => Ok(Self::Text),
            2 => Ok(Self::Secret),
            3 => Ok(Self::Url),
            4 => Ok(Self::Email),
            5 => Ok(Self::Date),
            6 => Ok(Self::Month),
            7 => Ok(Self::Notes),
            8 => Ok(Self::Number),
            _ => Err(Error::CorruptRecord),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormRecord {
    pub path: LockboxPath,
    pub name: String,
    pub type_id: FormTypeId,
    pub definition_alias: String,
    pub definition_revision: u32,
    pub values: Vec<FormFieldValue>,
}

#[derive(Debug, Clone)]
pub struct FormFieldValue {
    pub field_id: String,
    pub captured_label: String,
    pub kind: FormFieldKind,
    pub value: FormValue,
}

#[derive(Debug, Clone)]
pub enum FormValue {
    Normal(String),
    Secret(Arc<SecretString>),
}

impl FormValue {
    pub fn normal(value: impl Into<String>) -> Self {
        Self::Normal(value.into())
    }

    pub fn secret(value: SecretString) -> Self {
        Self::Secret(Arc::new(value))
    }

    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret(_))
    }
}

pub(crate) fn validate_form_alias(value: &str) -> Result<String> {
    validate_identifier(value, "form alias")
}

pub(crate) fn validate_form_field_id(value: &str) -> Result<String> {
    validate_identifier(value, "form field id")
}

pub(crate) fn validate_form_label(value: &str, description: &str) -> Result<String> {
    validate_text(value, description)?;
    Ok(value.to_string())
}

pub(crate) fn validate_form_description(value: &str) -> Result<String> {
    validate_text(value, "form description")?;
    Ok(value.to_string())
}

pub(crate) fn validate_form_record_name(value: &str) -> Result<String> {
    validate_text(value, "form record name")?;
    if value.trim().is_empty() {
        return Err(Error::InvalidInput(
            "form record name cannot be empty".to_string(),
        ));
    }
    Ok(value.to_string())
}

pub(crate) fn validate_form_value(kind: FormFieldKind, value: &FormValue) -> Result<()> {
    if kind.is_secret() != value.is_secret() {
        return Err(Error::InvalidOperation(
            "form field value sensitivity does not match the field definition".to_string(),
        ));
    }
    match value {
        FormValue::Normal(value) => validate_kind_text(kind, value),
        FormValue::Secret(value) => value.with_str(|value| validate_kind_text(kind, value))?,
    }
}

fn validate_identifier(value: &str, description: &str) -> Result<String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        || !value
            .chars()
            .all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric())
    {
        return Err(Error::InvalidInput(format!(
            "invalid {description}: {value}"
        )));
    }
    Ok(value.to_string())
}

fn validate_text(value: &str, description: &str) -> Result<()> {
    if value.len() > crate::constants::MAX_VARIABLE_VALUE_BYTES {
        return Err(Error::SecurityLimitExceeded(format!(
            "{description} exceeds {} bytes",
            crate::constants::MAX_VARIABLE_VALUE_BYTES
        )));
    }
    if value.contains('\0')
        || value.chars().any(|ch| {
            matches!(ch, '\u{0001}'..='\u{0008}' | '\u{000b}' | '\u{000c}' | '\u{000e}'..='\u{001f}' | '\u{007f}'..='\u{009f}')
        })
    {
        return Err(Error::InvalidInput(format!(
            "{description} contains unsupported control characters"
        )));
    }
    Ok(())
}

fn validate_kind_text(kind: FormFieldKind, value: &str) -> Result<()> {
    validate_text(value, "form field value")?;
    match kind {
        FormFieldKind::Url
            if !(value.is_empty()
                || value.starts_with("https://")
                || value.starts_with("http://")) =>
        {
            return Err(Error::InvalidInput(
                "url form field values must start with http:// or https://".to_string(),
            ));
        }
        FormFieldKind::Email
            if !value.is_empty()
                && (value.contains(char::is_whitespace)
                    || !value.contains('@')
                    || value.starts_with('@')
                    || value.ends_with('@')) =>
        {
            return Err(Error::InvalidInput(
                "email form field value is not a valid email address".to_string(),
            ));
        }
        FormFieldKind::Date if !value.is_empty() => validate_fixed_date(value, DateField::Date)?,
        FormFieldKind::Month if !value.is_empty() => validate_fixed_date(value, DateField::Month)?,
        FormFieldKind::Number if !value.is_empty() => {
            value.parse::<f64>().map_err(|_| {
                Error::InvalidInput("number form field value is not numeric".to_string())
            })?;
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum DateField {
    Date,
    Month,
}

impl DateField {
    fn expected_len(self) -> usize {
        match self {
            Self::Date => 10,
            Self::Month => 7,
        }
    }

    fn format_description(self) -> &'static str {
        match self {
            Self::Date => "date form field value must use YYYY-MM-DD",
            Self::Month => "month form field value must use YYYY-MM",
        }
    }

    fn field_name(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Month => "month",
        }
    }

    fn has_day(self) -> bool {
        matches!(self, Self::Date)
    }
}

fn validate_fixed_date(value: &str, field: DateField) -> Result<()> {
    let expected_len = field.expected_len();
    if value.len() != expected_len {
        return Err(Error::InvalidInput(field.format_description().to_string()));
    }
    let bytes = value.as_bytes();
    if bytes[4] != b'-'
        || (field.has_day() && bytes[7] != b'-')
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || (field.has_day() && idx == 7) || byte.is_ascii_digit())
    {
        return Err(Error::InvalidInput(field.format_description().to_string()));
    }
    let year = value[0..4].parse::<u16>().map_err(|_| {
        Error::InvalidInput(format!(
            "{} form field value year is invalid",
            field.field_name()
        ))
    })?;
    let month = value[5..7].parse::<u8>().map_err(|_| {
        Error::InvalidInput(format!(
            "{} form field value month is invalid",
            field.field_name()
        ))
    })?;
    if !(1..=12).contains(&month) {
        return Err(Error::InvalidInput(format!(
            "{} form field value month is invalid",
            field.field_name()
        )));
    }
    if field.has_day() {
        let day = value[8..10]
            .parse::<u8>()
            .map_err(|_| Error::InvalidInput("date form field value day is invalid".to_string()))?;
        if !(1..=days_in_month(year, month)).contains(&day) {
            return Err(Error::InvalidInput(
                "date form field value day is invalid".to_string(),
            ));
        }
    }
    Ok(())
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use super::{validate_form_value, FormFieldKind, FormValue};

    fn invalid_message(kind: FormFieldKind, value: &str) -> String {
        validate_form_value(kind, &FormValue::normal(value))
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn date_fields_require_calendar_date_format() {
        validate_form_value(FormFieldKind::Date, &FormValue::normal("2026-06-14")).unwrap();

        assert!(invalid_message(FormFieldKind::Date, "2026-06").contains("YYYY-MM-DD"));
        assert!(invalid_message(FormFieldKind::Date, "2026/06/14").contains("YYYY-MM-DD"));
        assert!(invalid_message(FormFieldKind::Date, "2026-13-14").contains("month is invalid"));
        assert!(invalid_message(FormFieldKind::Date, "2026-06-00").contains("day is invalid"));
        assert!(invalid_message(FormFieldKind::Date, "2026-02-29").contains("day is invalid"));
        validate_form_value(FormFieldKind::Date, &FormValue::normal("2024-02-29")).unwrap();
    }

    #[test]
    fn month_fields_require_year_month_format() {
        validate_form_value(FormFieldKind::Month, &FormValue::normal("2026-06")).unwrap();

        assert!(invalid_message(FormFieldKind::Month, "2026-06-14").contains("YYYY-MM"));
        assert!(invalid_message(FormFieldKind::Month, "2026/06").contains("YYYY-MM"));
        assert!(invalid_message(FormFieldKind::Month, "2026-13").contains("month is invalid"));
    }
}
