use crate::constants::{MAX_VARIABLE_NAME_BYTES, MAX_VARIABLE_VALUE_BYTES};
use crate::{Error, Result};

pub(crate) fn validate_permissions(permissions: u32) -> Result<u32> {
    if permissions <= 0o777 {
        Ok(permissions)
    } else {
        Err(Error::InvalidInput(format!(
            "permissions {permissions:o} include unsupported bits"
        )))
    }
}

pub(crate) fn validate_variable_name(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(invalid_variable_name(
            "a variable name is required; use a name such as TOKEN",
        ));
    }
    if name.len() > MAX_VARIABLE_NAME_BYTES {
        return Err(variable_name_too_long());
    }
    let canonical = if name.starts_with('/') {
        validate_variable_path(name)?;
        name.to_string()
    } else {
        validate_variable_component(name)?;
        format!("/{name}")
    };
    if canonical.len() > MAX_VARIABLE_NAME_BYTES {
        return Err(variable_name_too_long());
    }
    Ok(canonical)
}

fn validate_variable_path(path: &str) -> Result<()> {
    if path == "/" {
        return Err(invalid_variable_name(
            "'/' names the root inside the lockbox, not a variable; select a variable such as TOKEN or /production/TOKEN",
        ));
    }
    for component in path.split('/').skip(1) {
        validate_variable_component(component)?;
    }
    Ok(())
}

fn invalid_variable_name(reason: &str) -> Error {
    Error::InvalidPath(format!("invalid variable name: {reason}"))
}

fn variable_name_too_long() -> Error {
    invalid_variable_name(&format!(
        "name exceeds {MAX_VARIABLE_NAME_BYTES} bytes, including its leading '/'"
    ))
}

fn validate_variable_component(component: &str) -> Result<()> {
    if component.is_empty() {
        return Err(invalid_variable_name(
            "empty name component; remove repeated or trailing '/' separators",
        ));
    }
    if component == "." || component == ".." {
        return Err(invalid_variable_name(&format!(
            "component {component:?} is not a variable name"
        )));
    }
    let visible = component.strip_prefix('.').unwrap_or(component);
    for (index, ch) in visible.chars().enumerate() {
        if ch != '_' && !(ch.is_ascii_alphabetic() || index > 0 && ch.is_ascii_digit()) {
            let position = if index == 0 {
                "at the start of a name component"
            } else {
                "in a name component"
            };
            let allowed = if index == 0 {
                "an ASCII letter or '_'"
            } else {
                "an ASCII letter, digit or '_'"
            };
            return Err(invalid_variable_name(&format!(
                "character '{}' (U+{:04X}) {position}; use {allowed}",
                ch.escape_default(),
                ch as u32,
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_variable_value(value: &str) -> Result<String> {
    validate_variable_value_ref(value)?;
    Ok(value.to_string())
}

pub(crate) fn validate_variable_value_ref(value: &str) -> Result<()> {
    if value.len() > MAX_VARIABLE_VALUE_BYTES {
        return Err(Error::SecurityLimitExceeded(format!(
            "variable value exceeds {MAX_VARIABLE_VALUE_BYTES} bytes"
        )));
    }
    if value.contains('\0')
        || value.chars().any(|ch| {
            matches!(ch, '\u{0001}'..='\u{0008}' | '\u{000b}' | '\u{000c}' | '\u{000e}'..='\u{001f}' | '\u{007f}'..='\u{009f}')
        })
    {
        return Err(Error::InvalidInput(
            "variable value contains unsupported control characters".to_string(),
        ));
    }
    Ok(())
}
