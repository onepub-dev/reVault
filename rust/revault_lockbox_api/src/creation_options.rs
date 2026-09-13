//! Persistent archive choices and the credentials used at creation.

use crate::{Error, LockboxOptions, LockboxProtection, OwnerSigningKeyPair, Result};

/// Zstd compression level, from 1 (fastest) to 22 (strongest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZstdLevel(u8);

impl ZstdLevel {
    /// Validate a numeric Zstd level. Disabling compression is a separate choice.
    pub fn new(level: u8) -> Result<Self> {
        if (1..=22).contains(&level) {
            Ok(Self(level))
        } else {
            Err(Error::InvalidInput(
                "Zstd level must be between 1 and 22".into(),
            ))
        }
    }

    /// Return the numeric Zstd level.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl Default for ZstdLevel {
    fn default() -> Self {
        Self(3)
    }
}

/// Compression applied to newly written files and eligible metadata.
/// Secure variable and form pages retain their uncompressed encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// Store bytes without compression.
    None,
    /// Compress with Zstd, retaining raw bytes when compression is larger.
    Zstd {
        /// Numeric Zstd compression level.
        level: ZstdLevel,
    },
}

impl Default for Compression {
    fn default() -> Self {
        Self::Zstd {
            level: ZstdLevel::default(),
        }
    }
}

/// Persisted encryption mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    /// Content and metadata are publicly readable.
    None,
    /// Encrypt private pages with ChaCha20-Poly1305.
    ChaCha20Poly1305,
}

/// Persisted commit-signing requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningMode {
    /// Commits do not carry owner signatures.
    None,
    /// Commits require the established owner's hybrid signatures.
    /// For plaintext archives, creation, commit, and open also hash all current
    /// content to authenticate it without relying on page encryption.
    Owner,
}

/// Encryption choice and, when enabled, the material protecting the content key.
pub enum Encryption<'a> {
    /// Create an archive readable without credentials.
    None,
    /// Encrypt using the specified content-key protection.
    Encrypted(LockboxProtection<'a>),
}

/// Signing choice and, when enabled, the private key used for commits.
#[derive(Clone, Copy)]
pub enum Signing<'a> {
    /// Write unsigned commits.
    None,
    /// Sign commits with this owner's key; the caller retains ownership.
    Owner(&'a OwnerSigningKeyPair),
}

impl<'a> From<&'a OwnerSigningKeyPair> for Signing<'a> {
    fn from(key: &'a OwnerSigningKeyPair) -> Self {
        Self::Owner(key)
    }
}

/// Creation-time size-hiding policy, persisted in the authenticated header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SizePadding {
    /// Retain minimum page allocations and fixed-size private metadata pages.
    #[default]
    Default,
    /// Omit optional size padding. Stored lengths reveal more about small content.
    /// Headers, authentication tags, recovery metadata and free space still exist.
    None,
}

/// Independent creation choices. Runtime cache/worker settings are not persisted.
/// The default size padding is a privacy measure, not an 8 MiB archive minimum.
///
/// ```
/// use revault_lockbox_api::{Compression, Encryption, Lockbox, LockboxCreateOptions, Signing};
/// let options = LockboxCreateOptions {
///     compression: Compression::None,
///     ..LockboxCreateOptions::new(Encryption::None, Signing::None)
/// };
/// let archive = Lockbox::create_in_memory_with_options(options)?;
/// # Ok::<(), revault_lockbox_api::Error>(())
/// ```
pub struct LockboxCreateOptions<'a> {
    /// Encryption and content-key protection.
    pub encryption: Encryption<'a>,
    /// Commit signing and owner identity.
    pub signing: Signing<'a>,
    /// Compression algorithm and level, independent of protection choices.
    pub compression: Compression,
    /// Optional size-hiding padding; enabled by default.
    pub size_padding: SizePadding,
    /// Process-local cache and worker options.
    pub runtime: LockboxOptions,
}

impl<'a> LockboxCreateOptions<'a> {
    /// Choose encryption and signing explicitly; default to Zstd level 3.
    pub fn new(encryption: Encryption<'a>, signing: Signing<'a>) -> Self {
        Self {
            encryption,
            signing,
            compression: Compression::default(),
            size_padding: SizePadding::default(),
            runtime: LockboxOptions::default(),
        }
    }
}

/// Persisted choices, restored automatically when an archive is reopened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockboxFormatOptions {
    /// Page encryption mode.
    pub encryption: EncryptionMode,
    /// Required commit signatures.
    pub signing: SigningMode,
    /// Compression algorithm and level.
    pub compression: Compression,
    /// Creation-time size-hiding padding policy.
    pub size_padding: SizePadding,
}

/// Zero retains the historical v2 encoding and workload-dependent compression.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormatMode(pub(crate) u16);

impl FormatMode {
    pub(crate) fn new(options: LockboxFormatOptions) -> Self {
        let bits = (u16::from(options.size_padding == SizePadding::None) << 8)
            | u16::from(options.encryption == EncryptionMode::None)
            | (u16::from(options.signing == SigningMode::None) << 1)
            | match options.compression {
                Compression::None => 4,
                Compression::Zstd { level } => u16::from(level.get()) << 3,
            };
        Self(bits)
    }
    pub(crate) fn parse(bits: u16) -> Result<Self> {
        let encoding = bits & 0xff;
        if bits & !0x1ff != 0
            || (bits != 0
                && (encoding == 0
                    || if encoding & 4 != 0 {
                        encoding >> 3 != 0
                    } else {
                        !(1..=22).contains(&(encoding >> 3))
                    }))
        {
            return Err(Error::CorruptHeader);
        }
        Ok(Self(bits))
    }
    pub(crate) fn plaintext(self) -> bool {
        self.0 & 1 != 0
    }
    pub(crate) fn unpadded(self) -> bool {
        self.0 & 0x100 != 0
    }
    pub(crate) fn signed(self) -> bool {
        self.0 & 2 == 0
    }
    pub(crate) fn options(self) -> LockboxFormatOptions {
        LockboxFormatOptions {
            size_padding: if self.unpadded() {
                SizePadding::None
            } else {
                SizePadding::Default
            },
            encryption: if self.plaintext() {
                EncryptionMode::None
            } else {
                EncryptionMode::ChaCha20Poly1305
            },
            signing: if self.signed() {
                SigningMode::Owner
            } else {
                SigningMode::None
            },
            compression: if self.0 & 4 != 0 {
                Compression::None
            } else {
                Compression::Zstd {
                    level: ZstdLevel(if self.0 == 0 {
                        1
                    } else {
                        ((self.0 & 0xff) >> 3) as u8
                    }),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_bit_does_not_alias_codec_levels_or_legacy_mode() {
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for signing in [SigningMode::None, SigningMode::Owner] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    for compression in
                        std::iter::once(Compression::None).chain((1..=22).map(|level| {
                            Compression::Zstd {
                                level: ZstdLevel::new(level).unwrap(),
                            }
                        }))
                    {
                        let options = LockboxFormatOptions {
                            encryption,
                            signing,
                            compression,
                            size_padding,
                        };
                        let mode = FormatMode::new(options);
                        assert_eq!(FormatMode::parse(mode.0).unwrap().options(), options);
                        assert!(FormatMode::parse(mode.0 | 0x200).is_err());
                    }
                }
            }
        }
        assert_eq!(
            FormatMode::parse(0).unwrap().options().size_padding,
            SizePadding::Default
        );
        assert!(
            FormatMode::parse(0x100).is_err(),
            "do not create a second legacy encoding"
        );
    }

    #[test]
    fn content_pages_must_match_the_archive_padding_policy() {
        use crate::page::{PageObject, PageObjectKind};
        let objects = [PageObject::new(PageObjectKind::FileData, 1, vec![7; 29])];
        let mut options = LockboxFormatOptions {
            encryption: EncryptionMode::None,
            signing: SigningMode::None,
            compression: Compression::None,
            size_padding: SizePadding::Default,
        };
        let padded = FormatMode::new(options);
        options.size_padding = SizePadding::None;
        let compact = FormatMode::new(options);
        let id = crate::LockboxId::from_bytes([5; 16]);
        for mode in [padded, compact] {
            let len =
                crate::page::page_size_for_encoded_objects_with_format(&objects, mode).unwrap();
            let bytes =
                crate::page::encode_page_with_format(len, id, 1, 1, &[], &objects, mode).unwrap();
            assert!(crate::page::decode_page_with_format(&bytes, id, &[], mode).is_ok());
            assert!(crate::page::decode_page_with_format(
                &bytes,
                id,
                &[],
                if mode == padded { compact } else { padded }
            )
            .is_err());
        }
    }
}
