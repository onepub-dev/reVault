use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use unicode_normalization::UnicodeNormalization;

use crate::archive_reader::constants::{MAX_COMPONENT_BYTES, MAX_PATH_BYTES, MAX_PATH_DEPTH};
use crate::archive_reader::{Error, Result};

/// Canonical path for an directory, file or symlink entry inside a lockbox.
///
/// `LockboxPath` is distinct from `std::path::Path`, which represents a host
/// filesystem path. Lockbox paths always use `/` separators, are stored in
/// canonical Unicode form, and are validated against the lockbox path rules.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockboxPath(String);

impl LockboxPath {
    /// Validate and canonicalize a lockbox path.
    ///
    /// The root path `/` and trailing slash directory paths are allowed for
    /// APIs such as listing. File-specific APIs reject directory-only paths.
    ///
    /// Returns `Error::InvalidPath` if the path is relative, contains unsafe
    /// components, exceeds path limits, or contains unsupported characters.
    pub fn new(path: impl AsRef<str>) -> Result<Self> {
        Self::from_api(path.as_ref(), true)
    }

    /// Return the canonical string form of this lockbox path.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the parent lockbox path, or `None` for the lockbox root.
    pub fn parent(&self) -> Result<Option<Self>> {
        let Some(index) = self.0.rfind('/') else {
            return Err(Error::InvalidPath(
                "cannot find a parent: path inside the lockbox has no '/' separator".to_string(),
            ));
        };
        if index == 0 {
            return Ok(None);
        }
        Ok(Some(Self::from_api(&self.0[..index], false)?))
    }

    /// Return true when this path is below `directory`.
    pub fn is_descendant_of(&self, directory: &Self) -> bool {
        self != directory && self.0.starts_with(&directory.descendant_prefix())
    }

    /// Return true when this path is an immediate child of `directory`.
    pub fn is_direct_child_of(&self, directory: &Self) -> bool {
        if !self.is_descendant_of(directory) {
            return false;
        }
        let prefix = directory.descendant_prefix();
        let remainder = &self.0[prefix.len()..];
        !remainder.is_empty() && !remainder.contains('/')
    }

    pub(crate) fn descendant_prefix(&self) -> String {
        format!("{}/", self.0.trim_end_matches('/'))
    }

    pub(crate) fn from_api(path: &str, allow_dir: bool) -> Result<Self> {
        Ok(Self(canonicalize_api_path(path, allow_dir)?))
    }

    pub(crate) fn from_stored(path: &str, allow_dir: bool) -> Result<Self> {
        Ok(Self(canonicalize_stored_path(path, allow_dir)?))
    }

    pub(crate) fn as_file_path(&self) -> Result<&str> {
        validate_lockbox_path(&self.0, false)?;
        Ok(&self.0)
    }

    pub(crate) fn file_path(&self) -> Result<Self> {
        self.as_file_path()?;
        Ok(self.clone())
    }

    #[cfg(test)]
    pub(crate) fn from_unchecked_for_test(path: impl Into<String>) -> Self {
        Self(path.into())
    }
}

impl Borrow<str> for LockboxPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LockboxPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LockboxPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for LockboxPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<&str> for LockboxPath {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<LockboxPath> for &str {
    fn eq(&self, other: &LockboxPath) -> bool {
        *self == other.0
    }
}

impl TryFrom<&str> for LockboxPath {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for LockboxPath {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

pub(crate) fn canonicalize_api_path(path: &str, allow_dir: bool) -> Result<String> {
    if path.is_ascii() {
        validate_lockbox_path(path, allow_dir)?;
        return Ok(path.to_string());
    }
    let normalized = path.nfc().collect::<String>();
    validate_lockbox_path(&normalized, allow_dir)?;
    Ok(normalized)
}

pub(crate) fn canonicalize_stored_path(path: &str, allow_dir: bool) -> Result<String> {
    if path.is_ascii() {
        validate_lockbox_path(path, allow_dir)?;
        return Ok(path.to_string());
    }
    let normalized = path.nfc().collect::<String>();
    if normalized != path {
        return Err(Error::InvalidPath(
            "stored path inside the lockbox is not in canonical Unicode (NFC) form".to_string(),
        ));
    }
    validate_lockbox_path(path, allow_dir)?;
    Ok(normalized)
}

pub(crate) fn validate_stored_path(path: &str) -> Result<()> {
    LockboxPath::from_stored(path, false).map(|_| ())
}

pub(crate) fn validate_symlink_paths(link_path: &str, target_path: &str) -> Result<()> {
    LockboxPath::from_api(link_path, false)?;
    LockboxPath::from_api(target_path, false)?;
    Ok(())
}

fn validate_lockbox_path(path: &str, allow_dir: bool) -> Result<()> {
    if path.is_ascii() {
        return validate_ascii_lockbox_path(path, allow_dir);
    }

    let invalid = || invalid_lockbox_path(path, allow_dir);
    let path = if allow_dir && path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };

    if allow_dir && path == "/" {
        return Ok(());
    }

    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || !path.starts_with('/')
        || path.starts_with("//")
        || path.contains('\\')
        || path.contains('\0')
        || path.chars().any(is_forbidden_unicode)
        || path.contains(':')
    {
        return Err(invalid());
    }

    if !allow_dir && (path.len() == 1 || path.ends_with('/')) {
        return Err(invalid());
    }

    let mut depth = 0usize;
    for component in path.split('/').skip(1) {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.len() > MAX_COMPONENT_BYTES
        {
            return Err(invalid());
        }
        depth += 1;
        if depth > MAX_PATH_DEPTH {
            return Err(invalid());
        }
    }
    Ok(())
}

fn validate_ascii_lockbox_path(path: &str, allow_dir: bool) -> Result<()> {
    let invalid = || invalid_lockbox_path(path, allow_dir);
    let path = if allow_dir && path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };

    if allow_dir && path == "/" {
        return Ok(());
    }

    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || !path.starts_with('/')
        || path.starts_with("//")
        || path
            .as_bytes()
            .iter()
            .any(|byte| matches!(*byte, 0x00..=0x1f | 0x7f | b'\\' | b':'))
    {
        return Err(invalid());
    }

    if !allow_dir && (path.len() == 1 || path.ends_with('/')) {
        return Err(invalid());
    }

    let mut depth = 0usize;
    for component in path.split('/').skip(1) {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.len() > MAX_COMPONENT_BYTES
        {
            return Err(invalid());
        }
        depth += 1;
        if depth > MAX_PATH_DEPTH {
            return Err(invalid());
        }
    }
    Ok(())
}

// Called only after validation fails, keeping diagnostic work off the success path.
fn invalid_lockbox_path(path: &str, allow_dir: bool) -> Error {
    let path = if allow_dir && path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };
    let reason = if path.is_empty() {
        "a path inside the lockbox is required".to_string()
    } else if path.len() > MAX_PATH_BYTES {
        format!("path inside the lockbox exceeds {MAX_PATH_BYTES} bytes")
    } else if !path.starts_with('/') {
        "path inside the lockbox must start with '/' to identify the lockbox root; use a path such as /docs/file.txt".to_string()
    } else if path.starts_with("//") {
        "path inside the lockbox starts with repeated '/' separators; use a single leading '/'"
            .to_string()
    } else if let Some(ch) = path
        .chars()
        .find(|ch| matches!(ch, '\\' | ':') || is_forbidden_unicode(*ch))
    {
        format!(
            "path inside the lockbox contains unsupported character '{}' (U+{:04X})",
            ch.escape_default(),
            ch as u32
        )
    } else if !allow_dir && path == "/" {
        "this operation requires a file path inside the lockbox; '/' names its root directory"
            .to_string()
    } else if !allow_dir && path.ends_with('/') {
        "this operation requires a file path inside the lockbox; a trailing '/' denotes a directory"
            .to_string()
    } else {
        let mut reason = None;
        for (index, component) in path.split('/').skip(1).enumerate() {
            reason = if component.is_empty() {
                Some("path inside the lockbox contains an empty component; remove repeated '/' separators".to_string())
            } else if component == "." || component == ".." {
                Some(format!("path inside the lockbox contains unsupported component {component:?}; specify the entry from the lockbox root without '.' or '..'"))
            } else if component.len() > MAX_COMPONENT_BYTES {
                Some(format!(
                    "path component {} exceeds {MAX_COMPONENT_BYTES} bytes",
                    index + 1
                ))
            } else if index >= MAX_PATH_DEPTH {
                Some(format!(
                    "path inside the lockbox exceeds {MAX_PATH_DEPTH} components"
                ))
            } else {
                None
            };
            if reason.is_some() {
                break;
            }
        }
        reason.expect("invalid path must have a validation reason")
    };
    Error::InvalidPath(reason)
}

pub(super) fn is_forbidden_unicode(ch: char) -> bool {
    matches!(
        ch,
        '\u{0000}'..='\u{001f}'
            | '\u{007f}'..='\u{009f}'
            | '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{e0100}'..='\u{e01ef}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_paths_explain_the_actual_failure() {
        for (path, allow_dir, expected) in [
            ("", true, "a path inside the lockbox is required"),
            (
                "docs/file.txt",
                true,
                "must start with '/' to identify the lockbox root",
            ),
            ("//docs", true, "repeated '/' separators"),
            ("/", false, "requires a file path"),
            ("/docs/", false, "trailing '/' denotes a directory"),
            ("/docs//file", false, "empty component"),
            ("/docs/../file", false, "unsupported component \"..\""),
            ("/bad:name", false, "character ':' (U+003A)"),
            ("/bad\\name", false, "character '\\\\' (U+005C)"),
            ("/bad\nname", false, "character '\\n' (U+000A)"),
            ("/bad\u{200b}name", false, "character '\\u{200b}' (U+200B)"),
        ] {
            let error = validate_lockbox_path(path, allow_dir)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{path:?}: {error}");
            if !expected.contains("character") {
                assert!(!error.contains("character"), "{error}");
            }
        }
        let oversized = format!("/{}", "a".repeat(MAX_COMPONENT_BYTES + 1));
        assert!(LockboxPath::new(oversized)
            .unwrap_err()
            .to_string()
            .contains("component 1 exceeds"));
        let deep = "/a".repeat(MAX_PATH_DEPTH + 1);
        assert!(LockboxPath::new(deep)
            .unwrap_err()
            .to_string()
            .contains("components"));
        let long = format!("/{}", "a".repeat(MAX_PATH_BYTES));
        assert!(LockboxPath::new(long)
            .unwrap_err()
            .to_string()
            .contains("path inside the lockbox exceeds"));
    }

    #[test]
    fn symlink_validation_requires_lockbox_paths() {
        assert!(validate_symlink_paths("/links/current", "/docs/current").is_ok());

        for target in [
            "../outside",
            "/safe/../outside",
            "/C:/Users/target",
            "//server/share/target",
            "/safe\\target",
            "/safe/\0target",
        ] {
            assert!(
                matches!(
                    validate_symlink_paths("/links/current", target),
                    Err(Error::InvalidPath(_))
                ),
                "target should be rejected: {target:?}"
            );
        }

        assert!(matches!(
            validate_symlink_paths("/links/../current", "/docs/current"),
            Err(Error::InvalidPath(_))
        ));
    }
}
