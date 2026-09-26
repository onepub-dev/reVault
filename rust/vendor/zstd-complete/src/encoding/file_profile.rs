//! File-name and sample based classification for compression hints.

#[cfg(feature = "std")]
use crate::io::Read;
#[cfg(feature = "std")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
use super::util::{likely_composer_lockfile_text, likely_incompressible, likely_lockfile_text};

#[cfg(feature = "std")]
const FILE_TYPE_SAMPLE_BYTES: usize = 32 * 1024;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompressionFileProfile {
    None,
    CargoLock,
    ComposerLock,
    SmallTextLockfile,
    DependencyJsonLockfile,
}

impl CompressionFileProfile {
    pub(crate) const fn internal_hint_code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::CargoLock => 1,
            Self::ComposerLock => 2,
            Self::SmallTextLockfile => 3,
            Self::DependencyJsonLockfile => 4,
        }
    }

    pub(crate) const fn from_internal_hint_code(code: u8) -> Self {
        match code {
            1 => Self::CargoLock,
            2 => Self::ComposerLock,
            3 => Self::SmallTextLockfile,
            4 => Self::DependencyJsonLockfile,
            _ => Self::None,
        }
    }
}

#[cfg(feature = "std")]
mod known;

#[cfg(feature = "std")]
use known::{classify_file_name, file_name_matches_exact_or_prefixed};

#[cfg(feature = "std")]
fn likely_json_text_sample(data: &[u8]) -> bool {
    if !sample_looks_text(data) {
        return false;
    }

    let trimmed = data
        .iter()
        .skip_while(|&&byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
        .copied()
        .collect::<Vec<u8>>();
    let trimmed = trimmed.as_slice();
    if !matches!(trimmed.first(), Some(b'{') | Some(b'[')) {
        return false;
    }

    let window = &trimmed[..trimmed.len().min(2048)];
    window.contains(&b':') && window.iter().filter(|&&byte| byte == b'"').count() >= 4
}

#[cfg(feature = "std")]
fn sample_looks_text(data: &[u8]) -> bool {
    if data.len() < 16 {
        return false;
    }

    let step = (data.len() / 128).max(1);
    let mut printable = 0usize;
    let mut total = 0usize;
    for idx in (0..data.len()).step_by(step).take(128) {
        total += 1;
        let byte = data[idx];
        if byte == b'\n'
            || byte == b'\r'
            || byte == b'\t'
            || byte == b' '
            || byte.is_ascii_graphic()
        {
            printable += 1;
        }
    }

    printable * 100 >= total * 85
}

#[cfg(feature = "std")]
fn likely_dictionary_text_sample(data: &[u8]) -> bool {
    let mut package_markers = 0usize;
    let mut name_markers = 0usize;
    let mut version_markers = 0usize;
    let mut checksum_markers = 0usize;
    let mut composer_packages = false;
    let mut composer_require = 0usize;
    let mut composer_reference = 0usize;

    for line in data.split(|&byte| byte == b'\n').take(512) {
        let trimmed = line
            .iter()
            .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .copied()
            .collect::<Vec<u8>>();
        let trimmed = trimmed.as_slice();

        if trimmed.starts_with(b"[[package]]") {
            package_markers += 1;
        } else if trimmed.starts_with(b"name = \"") || trimmed.starts_with(br#""name": ""#) {
            name_markers += 1;
        } else if trimmed.starts_with(b"version = \"") || trimmed.starts_with(br#""version": ""#) {
            version_markers += 1;
        } else if trimmed.starts_with(b"checksum = \"") {
            checksum_markers += 1;
        } else if trimmed.starts_with(br#""packages": ["#) {
            composer_packages = true;
        } else if trimmed.starts_with(br#""require": {"#) {
            composer_require += 1;
        } else if trimmed.starts_with(br#""reference": ""#) {
            composer_reference += 1;
        }

        if package_markers >= 2
            && name_markers >= 2
            && version_markers >= 2
            && checksum_markers >= 2
        {
            return true;
        }
        if composer_packages
            && name_markers >= 2
            && version_markers >= 2
            && composer_require >= 2
            && composer_reference >= 2
        {
            return true;
        }
    }

    false
}

#[cfg(feature = "std")]
fn likely_config_text_sample(data: &[u8]) -> bool {
    if !sample_looks_text(data) {
        return false;
    }

    let mut score = 0usize;
    for line in data.split(|&byte| byte == b'\n').take(256) {
        let line = line
            .iter()
            .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .copied()
            .collect::<Vec<u8>>();
        let line = line.as_slice();
        if line.is_empty() {
            continue;
        }

        if (line.starts_with(b"[") && line.ends_with(b"]"))
            || (line.starts_with(b"<") && line.contains(&b'>'))
        {
            score += 2;
        } else if line.starts_with(b"- ") || line.starts_with(b"* ") {
            score += 1;
        } else if line.starts_with(b"FROM ")
            || line.starts_with(b"RUN ")
            || line.starts_with(b"ENV ")
            || line.starts_with(b"COPY ")
            || line.starts_with(b"WORKDIR ")
            || line.starts_with(b"ENTRYPOINT ")
            || line.starts_with(b"CMD ")
        {
            score += 2;
        } else if (line.windows(2).any(|pair| pair == b": ")
            && !line.windows(3).any(|pair| pair == b"://"))
            || line.contains(&b'=')
        {
            score += 1;
        }

        if score >= 4 {
            return true;
        }
    }

    false
}

#[cfg(feature = "std")]
fn likely_code_text_sample(data: &[u8]) -> bool {
    if !sample_looks_text(data) {
        return false;
    }

    let mut score = 0usize;
    for line in data.split(|&byte| byte == b'\n').take(256) {
        let line = line
            .iter()
            .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .copied()
            .collect::<Vec<u8>>();
        let line = line.as_slice();
        if line.is_empty() {
            continue;
        }

        if line.ends_with(b";") || line.ends_with(b"{") || line.ends_with(b"}") {
            score += 1;
        }
        if line
            .windows(2)
            .any(|pair| matches!(pair, b"::" | b"->" | b"=>" | b"//"))
        {
            score += 1;
        }
        if line.starts_with(b"fn ")
            || line.starts_with(b"pub ")
            || line.starts_with(b"let ")
            || line.starts_with(b"const ")
            || line.starts_with(b"var ")
            || line.starts_with(b"def ")
            || line.starts_with(b"class ")
            || line.starts_with(b"import ")
            || line.starts_with(b"package ")
            || line.starts_with(b"use ")
            || line.starts_with(b"#include")
            || line.starts_with(b"<?php")
        {
            score += 2;
        }

        if score >= 4 {
            return true;
        }
    }

    false
}

#[cfg(feature = "std")]
fn has_archive_signature(data: &[u8]) -> bool {
    data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
        || data.starts_with(&[0x1F, 0x8B])
        || data.starts_with(b"BZh")
        || data.starts_with(&[0xFD, b'7', b'z', b'X', b'Z', 0x00])
        || data.starts_with(&[0x28, 0xB5, 0x2F, 0xFD])
        || data.starts_with(&[b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C])
        || data.starts_with(b"Rar!\x1A\x07\x00")
        || data.starts_with(b"Rar!\x1A\x07\x01\x00")
        || (data.len() > 262 && &data[257..262] == b"ustar")
}

#[cfg(feature = "std")]
fn has_binary_signature(data: &[u8]) -> bool {
    data.starts_with(b"\x7FELF")
        || data.starts_with(b"MZ")
        || data.starts_with(b"\0asm")
        || data.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
        || data.starts_with(&[0xFE, 0xED, 0xFA, 0xCE])
        || data.starts_with(&[0xFE, 0xED, 0xFA, 0xCF])
        || data.starts_with(&[0xCE, 0xFA, 0xED, 0xFE])
        || data.starts_with(&[0xCF, 0xFA, 0xED, 0xFE])
        || data.starts_with(b"SQLite format 3\0")
        || data.starts_with(b"%PDF-")
}

#[cfg(feature = "std")]
fn compression_file_type_for_sample(sample: &[u8]) -> CompressionFileType {
    if sample.is_empty() {
        return CompressionFileType::Unknown;
    }
    if has_archive_signature(sample) {
        return CompressionFileType::ArchiveLike;
    }
    if has_binary_signature(sample) {
        return CompressionFileType::BinaryLike;
    }
    if likely_lockfile_text(sample)
        || likely_composer_lockfile_text(sample)
        || likely_dictionary_text_sample(sample)
    {
        return CompressionFileType::DictionaryText;
    }
    if likely_json_text_sample(sample) {
        return CompressionFileType::JsonText;
    }
    if likely_config_text_sample(sample) {
        return CompressionFileType::ConfigText;
    }
    if likely_code_text_sample(sample) {
        return CompressionFileType::CodeText;
    }
    if likely_incompressible(sample) {
        return CompressionFileType::BinaryLike;
    }
    CompressionFileType::Unknown
}

#[cfg(feature = "std")]
pub(crate) fn read_file_type_sample<R: Read>(source: &mut R) -> Vec<u8> {
    let mut sample = Vec::with_capacity(FILE_TYPE_SAMPLE_BYTES);
    let mut buffer = [0u8; 4096];
    while sample.len() < FILE_TYPE_SAMPLE_BYTES {
        let wanted = (FILE_TYPE_SAMPLE_BYTES - sample.len()).min(buffer.len());
        let read = source.read(&mut buffer[..wanted]).unwrap();
        if read == 0 {
            break;
        }
        sample.extend_from_slice(&buffer[..read]);
    }
    sample
}

/// Coarse file families used to choose the encoder's internal starting point.
///
/// This is intentionally small and stable. It is not an exposed bag of tuning knobs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CompressionFileType {
    Unknown,
    ArchiveLike,
    BinaryLike,
    CodeText,
    ConfigText,
    JsonText,
    DictionaryText,
}

#[cfg(feature = "std")]
pub fn compression_file_type_for_path(path: &Path) -> CompressionFileType {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    classify_file_name(&file_name.to_ascii_lowercase())
}

/// Classify using path first, then sample bytes if the path does not resolve to a known family.
#[cfg(feature = "std")]
pub fn compression_file_type_for_path_and_data(path: &Path, sample: &[u8]) -> CompressionFileType {
    let by_path = compression_file_type_for_path(path);
    if by_path != CompressionFileType::Unknown {
        return by_path;
    }
    compression_file_type_for_sample(sample)
}

#[cfg(feature = "std")]
pub(crate) fn compression_file_profile_for_path_and_data(
    path: &Path,
    sample: &[u8],
) -> CompressionFileProfile {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name_matches_exact_or_prefixed(&file_name, "cargo.lock") {
        CompressionFileProfile::CargoLock
    } else if file_name_matches_exact_or_prefixed(&file_name, "composer.lock") {
        CompressionFileProfile::ComposerLock
    } else if file_name_matches_exact_or_prefixed(&file_name, "poetry.lock")
        || file_name_matches_exact_or_prefixed(&file_name, "pubspec.lock")
    {
        CompressionFileProfile::SmallTextLockfile
    } else if file_name_matches_exact_or_prefixed(&file_name, "package-lock.json")
        || file_name_matches_exact_or_prefixed(&file_name, "pipfile.lock")
        || super::util::likely_dependency_json_lockfile_text(sample)
    {
        CompressionFileProfile::DependencyJsonLockfile
    } else {
        CompressionFileProfile::None
    }
}

#[cfg(all(test, feature = "std"))]
mod tests;
