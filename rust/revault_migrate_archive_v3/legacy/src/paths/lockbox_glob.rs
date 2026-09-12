use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use unicode_normalization::UnicodeNormalization;

use super::lockbox_path::is_forbidden_unicode;
use crate::constants::MAX_PATH_BYTES;
use crate::{Error, Result};

/// Validated glob pattern for matching paths inside a lockbox.
///
/// Patterns are relative to a listing root. `*` and `?` match within one path
/// component, while `**` matches across path components.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockboxGlob(String);

impl LockboxGlob {
    /// Validate and canonicalize a lockbox glob pattern.
    pub fn new(pattern: impl AsRef<str>) -> Result<Self> {
        let pattern = pattern.as_ref();
        if pattern.is_empty()
            || pattern.len() > MAX_PATH_BYTES
            || pattern.starts_with('/')
            || pattern.starts_with("//")
            || pattern.contains('\\')
            || pattern.contains('\0')
            || pattern.contains(':')
            || pattern.chars().any(is_forbidden_unicode)
            || pattern
                .split('/')
                .any(|component| component.is_empty() || component == "." || component == "..")
        {
            return Err(Error::InvalidPath(pattern.to_string()));
        }
        if pattern.is_ascii() {
            Ok(Self(pattern.to_string()))
        } else {
            Ok(Self(pattern.nfc().collect()))
        }
    }

    /// Return the canonical pattern.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return whether this pattern matches `path`.
    pub fn matches(&self, path: impl AsRef<str>) -> bool {
        let pattern_parts = self.0.split('/').collect::<Vec<_>>();
        let path_parts = path.as_ref().split('/').collect::<Vec<_>>();
        match_parts(&pattern_parts, &path_parts)
    }
}

impl AsRef<str> for LockboxGlob {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for LockboxGlob {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for LockboxGlob {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for LockboxGlob {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for LockboxGlob {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for LockboxGlob {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

fn match_parts(pattern: &[&str], path: &[&str]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    if pattern[0] == "**" {
        return match_parts(&pattern[1..], path)
            || (!path.is_empty() && match_parts(pattern, &path[1..]));
    }
    if path.is_empty() {
        return false;
    }
    match_component(pattern[0], path[0]) && match_parts(&pattern[1..], &path[1..])
}

fn match_component(pattern: &str, text: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let text = text.chars().collect::<Vec<_>>();
    let mut pattern_index = 0usize;
    let mut text_index = 0usize;
    let mut star = None;
    let mut star_text = 0usize;

    while text_index < text.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == '?' || pattern[pattern_index] == text[text_index])
        {
            pattern_index += 1;
            text_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star = Some(pattern_index);
            pattern_index += 1;
            star_text = text_index;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            star_text += 1;
            text_index = star_text;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_star_matches_path_components() {
        let glob = LockboxGlob::new("docs/**/*.txt").unwrap();
        assert!(glob.matches("docs/archive/readme.txt"));
        assert!(glob.matches("docs/readme.txt"));
        assert!(!glob.matches("images/readme.txt"));
    }

    #[test]
    fn rejects_absolute_and_traversal_patterns() {
        for pattern in ["/docs/*", "docs/../*", "docs//*.txt", "docs\\*.txt"] {
            assert!(LockboxGlob::new(pattern).is_err());
        }
    }
}
