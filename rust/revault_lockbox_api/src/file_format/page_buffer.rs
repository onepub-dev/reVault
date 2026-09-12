use crate::secret_vec::{secure_read_access, SecureVec};
use crate::{Error, Result};
use std::ops::{Deref, DerefMut};
use zeroize::Zeroize;

// Vec<T>::zeroize first wipes every element separately and then wipes the
// entire allocation again. For bytes, slice zeroization provides the same
// volatile-write guarantees with one barrier per slice. Cover spare capacity
// too, since truncation may have left sensitive bytes there. Retain the length.
pub(crate) fn zeroize_bytes(bytes: &mut Vec<u8>) {
    bytes.as_mut_slice().zeroize();
    bytes.spare_capacity_mut().zeroize();
}

pub(crate) struct ZeroizingBytes(Vec<u8>);

impl ZeroizingBytes {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl Deref for ZeroizingBytes {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZeroizingBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for ZeroizingBytes {
    fn drop(&mut self) {
        zeroize_bytes(&mut self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_wiping_handles_empty_and_partial_allocations() {
        for len in [0, 1, 7, 8, 31, 1024] {
            let mut bytes = vec![0xa5; 2048];
            bytes.resize(bytes.capacity(), 0xa5);
            bytes.truncate(len);
            let capacity = bytes.capacity();
            zeroize_bytes(&mut bytes);
            assert_eq!(bytes, vec![0; len]);
            assert_eq!(bytes.capacity(), capacity);
            // SAFETY: all capacity was initialized above; truncation does not
            // deallocate it and zeroize_bytes writes every spare byte as well.
            for byte in bytes.spare_capacity_mut() {
                assert_eq!(unsafe { byte.assume_init() }, 0);
            }
        }
        zeroize_bytes(&mut Vec::new());
    }
}

pub(crate) trait PageBuffer: Sized {
    fn truncate(&mut self, len: usize) -> Result<()>;
    fn try_clone_range(&self, offset: usize, len: usize) -> Result<Self>;
    fn with_bytes<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> Result<R>;
    fn with_mut_bytes<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> Result<R>;
}

impl PageBuffer for Vec<u8> {
    fn truncate(&mut self, len: usize) -> Result<()> {
        Vec::truncate(self, len);
        Ok(())
    }

    fn try_clone_range(&self, offset: usize, len: usize) -> Result<Self> {
        let end = offset.checked_add(len).ok_or(Error::CorruptRecord)?;
        let range = self.get(offset..end).ok_or(Error::CorruptRecord)?;
        Ok(range.to_vec())
    }

    fn with_bytes<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> Result<R> {
        Ok(f(self))
    }

    fn with_mut_bytes<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> Result<R> {
        Ok(f(self))
    }
}

impl PageBuffer for SecureVec {
    fn truncate(&mut self, len: usize) -> Result<()> {
        SecureVec::truncate(self, len)?;
        Ok(())
    }

    fn try_clone_range(&self, offset: usize, len: usize) -> Result<Self> {
        Ok(SecureVec::try_clone_range(self, offset, len)?)
    }

    fn with_bytes<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> Result<R> {
        secure_read_access(|access| self.with_bytes_in(access, f)).map_err(Into::into)
    }

    fn with_mut_bytes<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> Result<R> {
        SecureVec::with_mut_bytes(self, f).map_err(Into::into)
    }
}
