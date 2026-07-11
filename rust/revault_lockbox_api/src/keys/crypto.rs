use chacha20poly1305::aead::{Aead, AeadInOut, KeyInit, Payload};
use chacha20poly1305::Tag;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use getrandom::fill as getrandom;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::secret_vec::SecureVec;
use crate::{Error, Result};

pub(crate) fn seal_with_random_nonce(
    payload: &[u8],
    key: &[u8],
    aad: &[u8],
) -> Result<([u8; 12], Vec<u8>)> {
    let mut content_key = derive_content_key(key);
    let cipher = ChaCha20Poly1305::new(&Key::from(content_key));
    content_key.zeroize();
    let mut nonce = [0u8; 12];
    getrandom(&mut nonce).map_err(|err| Error::Io(err.to_string()))?;
    let ciphertext = cipher
        .encrypt(&Nonce::from(nonce), Payload { msg: payload, aad })
        .map_err(|_| Error::SecurityLimitExceeded("encryption failed".to_string()))?;
    Ok((nonce, ciphertext))
}

pub(crate) fn open_with_nonce(
    payload: &[u8],
    key: &[u8],
    nonce: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    if nonce.len() != 12 {
        return Err(Error::CorruptRecord);
    }
    let mut content_key = derive_content_key(key);
    let cipher = ChaCha20Poly1305::new(&Key::from(content_key));
    content_key.zeroize();
    let nonce = Nonce::try_from(nonce).map_err(|_| Error::CorruptRecord)?;
    cipher
        .decrypt(&nonce, Payload { msg: payload, aad })
        .map_err(|_| Error::InvalidKey)
}

pub(crate) fn open_with_content_key_secure(
    payload: &mut SecureVec,
    content_key: &[u8; 32],
    nonce: &[u8],
    aad: &[u8],
) -> Result<()> {
    if nonce.len() != 12 {
        return Err(Error::CorruptRecord);
    }
    if payload.len() < 16 {
        return Err(Error::CorruptRecord);
    }
    let cipher = ChaCha20Poly1305::new(&Key::from(*content_key));
    let nonce = Nonce::try_from(nonce).map_err(|_| Error::CorruptRecord)?;
    payload.with_mut_bytes(|bytes| {
        let tag_offset = bytes.len() - 16;
        let (message, tag_bytes) = bytes.split_at_mut(tag_offset);
        let tag = Tag::try_from(&*tag_bytes).map_err(|_| Error::CorruptRecord)?;
        cipher
            .decrypt_inout_detached(&nonce, aad, message.into(), &tag)
            .map_err(|_| Error::InvalidKey)
    })??;
    payload.truncate(payload.len() - 16)?;
    Ok(())
}

pub(crate) fn seal_with_content_key_secure(
    payload: &mut SecureVec,
    content_key: &[u8; 32],
    aad: &[u8],
) -> Result<[u8; 12]> {
    let cipher = ChaCha20Poly1305::new(&Key::from(*content_key));
    let mut nonce = [0u8; 12];
    getrandom(&mut nonce).map_err(|err| Error::Io(err.to_string()))?;
    let tag = payload.with_mut_bytes(|bytes| {
        cipher
            .encrypt_inout_detached(&Nonce::from(nonce), aad, bytes.into())
            .map_err(|_| Error::SecurityLimitExceeded("encryption failed".to_string()))
    })?;
    let tag = tag?;
    payload.try_extend_from_slice(&tag)?;
    Ok(nonce)
}

pub(crate) fn derive_page_content_key(key: &[u8]) -> [u8; 32] {
    derive_content_key(key)
}

pub(crate) fn strong_checksum(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lockbox-v1-public-checksum/sha256");
    hasher.update((data.len() as u64).to_le_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

fn derive_content_key(key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lockbox-v1-content-key/chacha20poly1305");
    hasher.update((key.len() as u64).to_le_bytes());
    hasher.update(key);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_with_random_nonce_uses_unique_nonce() {
        let key = b"secret";
        let aad = b"test-aad";
        let (first_nonce, first) = seal_with_random_nonce(b"payload", key, aad).unwrap();
        let (second_nonce, second) = seal_with_random_nonce(b"payload", key, aad).unwrap();

        assert_ne!(first, second);
        assert_ne!(first_nonce, second_nonce);
        assert_eq!(
            open_with_nonce(&first, key, &first_nonce, aad).unwrap(),
            b"payload"
        );
        assert_eq!(
            open_with_nonce(&second, key, &second_nonce, aad).unwrap(),
            b"payload"
        );
    }
}
