use std::sync::Arc;

use super::validation::validate_text;
use crate::{Error, Result, SecretString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Validation and sensitivity category for a form field.
pub enum FormFieldKind {
    /// Single-line plain text.
    Text,
    /// UTF-8 text retained in secure memory and redacted from metadata views.
    Secret,
    /// Absolute URL text.
    Url,
    /// Email-address text.
    Email,
    /// Calendar date in `YYYY-MM-DD` form.
    Date,
    /// Calendar month in `YYYY-MM` form.
    Month,
    /// Multi-line plain text.
    Notes,
    /// Numeric text accepted by the form validator.
    Number,
}

impl FormFieldKind {
    /// Returns whether values of this kind must use secure memory.
    pub fn is_secret(self) -> bool {
        matches!(self, Self::Secret)
    }

    pub(crate) fn validate_value(self, value: &FormValue) -> Result<()> {
        if self.is_secret() != value.is_secret() {
            return Err(Error::InvalidOperation(
                "form field value sensitivity does not match the field definition".to_string(),
            ));
        }
        match value {
            FormValue::Normal(value) => self.validate_text(value),
            FormValue::Secret(value) => value.with_str(|value| self.validate_text(value))?,
        }
    }

    fn validate_text(self, value: &str) -> Result<()> {
        validate_text(value, "form field value")?;
        match self {
            Self::Url
                if !(value.is_empty()
                    || value.starts_with("https://")
                    || value.starts_with("http://")) =>
            {
                return Err(Error::InvalidInput(
                    "url form field values must start with http:// or https://".to_string(),
                ));
            }
            Self::Email
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
            Self::Date if !value.is_empty() => DateField::Date.validate(value)?,
            Self::Month if !value.is_empty() => DateField::Month.validate(value)?,
            Self::Number if !value.is_empty() => {
                value.parse::<f64>().map_err(|_| {
                    Error::InvalidInput("number form field value is not numeric".to_string())
                })?;
            }
            _ => {}
        }
        Ok(())
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
/// One value stored in a form record with captured display metadata.
pub struct FormFieldValue {
    /// Case-sensitive machine-readable field identifier.
    pub field_id: String,
    /// User-facing label captured when the value was set.
    pub captured_label: String,
    /// Field kind used to validate the value.
    pub kind: FormFieldKind,
    /// Normal or secure value payload.
    pub value: FormValue,
}

#[derive(Debug, Clone)]
/// Sensitivity-aware payload stored by a form field.
pub enum FormValue {
    /// Non-secret UTF-8 value held as an ordinary string.
    Normal(String),
    /// Secret UTF-8 value held in shared secure memory.
    Secret(Arc<SecretString>),
}

impl FormValue {
    /// Creates a non-secret form value.
    pub fn normal(value: impl Into<String>) -> Self {
        Self::Normal(value.into())
    }

    /// Moves a secure string into a secret form value.
    pub fn secret(value: SecretString) -> Self {
        Self::Secret(Arc::new(value))
    }

    /// Returns whether this value contains secure secret text.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret(_))
    }
}

#[derive(Clone, Copy)]
enum DateField {
    Date,
    Month,
}

impl DateField {
    fn validate(self, value: &str) -> Result<()> {
        if value.len() != self.expected_len() {
            return Err(Error::InvalidInput(self.format_description().to_string()));
        }
        let bytes = value.as_bytes();
        if bytes[4] != b'-'
            || (self.has_day() && bytes[7] != b'-')
            || !bytes.iter().enumerate().all(|(idx, byte)| {
                idx == 4 || (self.has_day() && idx == 7) || byte.is_ascii_digit()
            })
        {
            return Err(Error::InvalidInput(self.format_description().to_string()));
        }
        let year = value[0..4].parse::<u16>().map_err(|_| {
            Error::InvalidInput(format!(
                "{} form field value year is invalid",
                self.field_name()
            ))
        })?;
        let month = value[5..7].parse::<u8>().map_err(|_| {
            Error::InvalidInput(format!(
                "{} form field value month is invalid",
                self.field_name()
            ))
        })?;
        if !(1..=12).contains(&month) {
            return Err(Error::InvalidInput(format!(
                "{} form field value month is invalid",
                self.field_name()
            )));
        }
        if self.has_day() {
            let day = value[8..10].parse::<u8>().map_err(|_| {
                Error::InvalidInput("date form field value day is invalid".to_string())
            })?;
            if !(1..=Self::days_in_month(year, month)).contains(&day) {
                return Err(Error::InvalidInput(
                    "date form field value day is invalid".to_string(),
                ));
            }
        }
        Ok(())
    }

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

    fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    fn is_leap_year(year: u16) -> bool {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    }
}

#[cfg(test)]
mod tests {
    use super::{FormFieldKind, FormValue};

    fn invalid_message(kind: FormFieldKind, value: &str) -> String {
        kind.validate_value(&FormValue::normal(value))
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn date_fields_require_calendar_date_format() {
        FormFieldKind::Date
            .validate_value(&FormValue::normal("2026-06-14"))
            .unwrap();

        assert!(invalid_message(FormFieldKind::Date, "2026-06").contains("YYYY-MM-DD"));
        assert!(invalid_message(FormFieldKind::Date, "2026/06/14").contains("YYYY-MM-DD"));
        assert!(invalid_message(FormFieldKind::Date, "2026-13-14").contains("month is invalid"));
        assert!(invalid_message(FormFieldKind::Date, "2026-06-00").contains("day is invalid"));
        assert!(invalid_message(FormFieldKind::Date, "2026-02-29").contains("day is invalid"));
        FormFieldKind::Date
            .validate_value(&FormValue::normal("2024-02-29"))
            .unwrap();
    }

    #[test]
    fn month_fields_require_year_month_format() {
        FormFieldKind::Month
            .validate_value(&FormValue::normal("2026-06"))
            .unwrap();

        assert!(invalid_message(FormFieldKind::Month, "2026-06-14").contains("YYYY-MM"));
        assert!(invalid_message(FormFieldKind::Month, "2026/06").contains("YYYY-MM"));
        assert!(invalid_message(FormFieldKind::Month, "2026-13").contains("month is invalid"));
    }
}
