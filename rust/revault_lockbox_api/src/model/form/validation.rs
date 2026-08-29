use crate::{Error, Result};

pub(super) fn validate_identifier(value: &str, description: &str) -> Result<String> {
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

pub(super) fn validate_text(value: &str, description: &str) -> Result<()> {
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
