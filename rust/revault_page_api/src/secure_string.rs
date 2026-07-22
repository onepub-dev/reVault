use std::{env, ffi::OsString, fmt, str};

use zeroize::Zeroize;

use crate::{
    error::{Error, Result},
    secure_access::{read_access, SecureReadAccess},
    secure_vec::SecureVec,
};

/// UTF-8 secret stored in guarded, zeroizing memory.
///
/// The value intentionally implements neither `Display` nor direct string
/// access. Use [`SecureString::with_str`] or [`SecureString::with_bytes`] and do
/// not retain copies beyond the callback.
pub struct SecureString {
    bytes: SecureVec,
}

impl SecureString {
    /// Creates an empty secure string without allocating secret pages.
    pub fn new() -> Self {
        Self {
            bytes: SecureVec::new(),
        }
    }

    /// Moves bytes into secure memory and zeroizes the input vector.
    ///
    /// UTF-8 is validated when text access is requested. Prefer
    /// [`SecureString::try_from_utf8`] when invalid input should fail eagerly.
    pub fn try_from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            bytes: SecureVec::try_from_vec(bytes)?,
        })
    }

    /// Copies bytes into a new secure string.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            bytes: SecureVec::try_from_slice(bytes)?,
        })
    }

    /// Wraps an existing secure byte vector without copying it.
    pub fn from_secure_vec(bytes: SecureVec) -> Self {
        Self { bytes }
    }

    /// Validates UTF-8, moves the bytes into secure memory, and zeroizes input.
    pub fn try_from_utf8(bytes: Vec<u8>) -> Result<Self> {
        str::from_utf8(&bytes).map_err(|_| Error::InvalidUtf8)?;
        Self::try_from_bytes(bytes)
    }

    /// Reads an environment variable directly into a secure string when set.
    pub fn try_from_env(name: &str) -> Result<Option<Self>> {
        env::var_os(name).map(os_string_to_secure).transpose()
    }

    /// Converts an operating-system string into a secure UTF-8 representation.
    pub fn try_from_os_string(value: OsString) -> Result<Self> {
        os_string_to_secure(value)
    }

    /// Creates an independent secure copy of this string.
    pub fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            bytes: self.bytes.try_clone()?,
        })
    }

    /// Exposes valid UTF-8 text only for the duration of `f`.
    pub fn with_str<R>(&self, f: impl FnOnce(&str) -> R) -> Result<R> {
        read_access(|access| self.with_str_in(access, f))
    }

    /// Exposes valid UTF-8 text using an existing secure read scope.
    pub fn with_str_in<R>(
        &self,
        access: &SecureReadAccess<'_>,
        f: impl FnOnce(&str) -> R,
    ) -> Result<R> {
        self.bytes.with_bytes_in(access, |bytes| {
            let text = str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?;
            Ok(f(text))
        })?
    }

    /// Exposes the encoded bytes only for the duration of `f`.
    pub fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R> {
        self.bytes.with_bytes(f)
    }

    /// Exposes encoded bytes using an existing secure read scope.
    pub fn with_bytes_in<R>(
        &self,
        access: &SecureReadAccess<'_>,
        f: impl FnOnce(&[u8]) -> R,
    ) -> Result<R> {
        self.bytes.with_bytes_in(access, f)
    }

    /// Appends this value to `target` without copying through normal heap memory.
    pub fn append_to_secure_vec(&self, target: &mut SecureVec) -> Result<()> {
        target.try_extend_from_secure(&self.bytes)
    }

    /// Appends one byte to the secure string.
    ///
    /// This low-level operation does not validate that the resulting value is
    /// UTF-8; subsequent text access will fail if the sequence is invalid.
    pub fn try_push_byte(&mut self, byte: u8) -> Result<()> {
        self.bytes.try_push(byte)
    }

    /// Appends bytes without intermediate normal-memory allocation.
    pub fn try_extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
        self.bytes.try_extend_from_slice(bytes)
    }

    /// Encodes and appends one Unicode scalar value.
    pub fn try_push_utf8_char(&mut self, ch: char) -> Result<()> {
        let mut encoded = [0u8; 4];
        let text = ch.encode_utf8(&mut encoded);
        let result = self.bytes.try_extend_from_slice(text.as_bytes());
        encoded.zeroize();
        result
    }

    /// Removes, zeroes, and returns the final encoded byte, if present.
    pub fn try_pop_byte(&mut self) -> Result<Option<u8>> {
        self.bytes.try_pop()
    }

    /// Returns `true` when the string contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Overwrites the allocation and resets the string to empty.
    pub fn zeroize(&mut self) -> Result<()> {
        self.bytes.zeroize()
    }
}

impl Default for SecureString {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureString")
            .field("len", &self.bytes.len())
            .field("redacted", &true)
            .finish()
    }
}

impl PartialEq for SecureString {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for SecureString {}

#[cfg(unix)]
fn os_string_to_secure(value: OsString) -> Result<SecureString> {
    use std::os::unix::ffi::OsStringExt;

    SecureString::try_from_bytes(value.into_vec())
}

#[cfg(windows)]
fn os_string_to_secure(value: OsString) -> Result<SecureString> {
    use std::os::windows::ffi::OsStrExt;

    let mut secret = SecureString::new();
    for ch in char::decode_utf16(value.encode_wide()).flatten() {
        secret.try_push_utf8_char(ch)?;
    }
    Ok(secret)
}

#[cfg(not(any(unix, windows)))]
fn os_string_to_secure(value: OsString) -> Result<SecureString> {
    let mut secret = SecureString::new();
    for ch in value.to_string_lossy().chars() {
        secret.try_push_utf8_char(ch)?;
    }
    Ok(secret)
}
