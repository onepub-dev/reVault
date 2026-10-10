//! Alias grammar uses Unicode 17.0 data, pinned through ICU4X 2.3.0.
use icu_normalizer::ComposingNormalizer;
use icu_properties::props::{DefaultIgnorableCodePoint, GeneralCategory, GeneralCategoryGroup};
use icu_properties::{CodePointMapData, CodePointSetData};
use revault_lockbox_api::{Error, Result};

/// Normalize a partial alias for case-sensitive completion matching.
pub fn normalize_lockbox_alias_prefix(value: &str) -> String {
    ComposingNormalizer::new_nfc().normalize(value).into_owned()
}

fn permitted(ch: char, first: bool, follows_base: bool) -> bool {
    if CodePointSetData::new::<DefaultIgnorableCodePoint>().contains(ch) {
        return false;
    }
    let category = CodePointMapData::<GeneralCategory>::new().get(ch);
    GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category)
        || ch == '_'
        || (!first
            && (matches!(ch, '.' | '-')
                || (follows_base && GeneralCategoryGroup::Mark.contains(category))))
}

fn is_base(ch: char) -> bool {
    let category = CodePointMapData::<GeneralCategory>::new().get(ch);
    GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category)
        || GeneralCategoryGroup::Mark.contains(category)
}

fn repair(value: &str) -> String {
    let mut result = String::new();
    let mut follows_base = false;
    for (index, ch) in value.chars().enumerate() {
        let ch = if permitted(ch, index == 0, follows_base) {
            ch
        } else {
            '_'
        };
        result.push(ch);
        follows_base = is_base(ch);
    }
    if result.is_empty() {
        result.push('_');
    }
    result
}

/// Derive an alias from a filename stem: NFC, then one underscore per invalid
/// character, including invalid positional characters. Never truncate a name.
pub fn derive_lockbox_alias_name(stem: &str) -> Result<String> {
    let name = repair(&normalize_lockbox_alias_prefix(stem));
    if name.len() > 128 {
        return Err(Error::InvalidInput("derived lockbox alias exceeds 128 UTF-8 bytes after NFC normalization; choose a shorter name with --alias".into()));
    }
    Ok(name)
}

/// Validate a new alias and return its canonical NFC spelling.
pub fn normalize_lockbox_alias_name(value: &str) -> Result<String> {
    let name = normalize_lockbox_alias_prefix(value);
    if name.len() > 128 {
        return Err(Error::InvalidInput("lockbox alias exceeds 128 UTF-8 bytes after NFC normalization; choose a shorter alias (for example, project)".into()));
    }
    let repaired = repair(&name);
    if repaired != name {
        return Err(Error::InvalidInput(format!("invalid lockbox alias; use {repaired:?}. Start with a Unicode letter, number or underscore; then use letters, numbers, attached combining marks, dot, underscore or hyphen. Invisible characters are not allowed")));
    }
    Ok(name)
}

// Earlier development builds accepted leading ASCII hyphens. Keep those
// records readable, resolvable and removable, but do not create new ones.
pub(super) fn lookup_name(value: &str) -> Result<String> {
    let name = normalize_lockbox_alias_prefix(value);
    if !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-'))
    {
        return Ok(name);
    }
    normalize_lockbox_alias_name(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_grammar_and_nfc_are_shared() {
        for name in [
            "développeur",
            "東京.開発",
            "١٢٣",
            "_private",
            "dev.",
            "कर्म",
            "a\u{0301}",
        ] {
            assert!(normalize_lockbox_alias_name(name).is_ok(), "{name:?}");
        }
        assert_eq!(normalize_lockbox_alias_name("de\u{0301}v").unwrap(), "dév");
        for name in [
            "",
            ".",
            "..",
            "-option",
            ".hidden",
            "\u{0301}",
            "_\u{0301}",
            "a-\u{0301}",
            "a b",
            "a\n",
            "a/b",
            "a@b",
            "a$b",
            "a;",
            "a\u{034f}",
            "a\u{200d}",
            "a\u{202e}",
            "a\u{fe0f}",
            "\u{115f}",
        ] {
            assert!(normalize_lockbox_alias_name(name).is_err(), "{name:?}");
        }
        assert!(lookup_name("-legacy").is_ok());
        assert!(normalize_lockbox_alias_name("-legacy").is_err());
    }
    #[test]
    fn derivation_repairs_each_character_and_checks_normalized_bytes() {
        assert_eq!(
            derive_lockbox_alias_name("-a  /b\u{200d}").unwrap(),
            "_a___b_"
        );
        assert_eq!(derive_lockbox_alias_name(".\u{0301}x").unwrap(), "__x");
        assert_eq!(derive_lockbox_alias_name("de\u{0301}v").unwrap(), "dév");
        assert_eq!(derive_lockbox_alias_name("").unwrap(), "_");
        assert_eq!(
            normalize_lockbox_alias_name(&format!("{}a", "é".repeat(63)))
                .unwrap()
                .len(),
            127
        );
        assert_eq!(
            normalize_lockbox_alias_name(&"e\u{0301}".repeat(64))
                .unwrap()
                .len(),
            128
        );
        assert!(normalize_lockbox_alias_name(&format!("{}a", "é".repeat(64))).is_err());
        assert!(normalize_lockbox_alias_name(&"é".repeat(65)).is_err());
        assert!(derive_lockbox_alias_name(&"é".repeat(65)).is_err());
    }
}
