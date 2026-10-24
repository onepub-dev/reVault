use argon2::{Algorithm, Argon2, Params, Version};

use crate::archive_reader::secret_vec::SecretString;
use crate::archive_reader::{Error, Result};

pub(crate) fn derive_key_from_password(password: &SecretString, salt: &[u8]) -> Result<[u8; 32]> {
    password.with_bytes(|bytes| derive_key_from_password_bytes(bytes, salt))?
}

pub(crate) fn derive_key_from_password_bytes(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    if salt.len() < 16 {
        return Err(Error::InvalidInput(
            "Argon2id salt must be at least 16 bytes".to_string(),
        ));
    }
    let params = Params::new(64 * 1024, 3, 1, Some(32))
        .map_err(|err| Error::SecurityLimitExceeded(err.to_string()))?;
    // The allocating Argon2 convenience API deallocates its large block arena
    // without wiping it. Own the unchanged algorithm's working memory here so
    // password-derived blocks are cleared on success, error, and unwind.
    let count = params.block_count();
    let mut memory = zeroize::Zeroizing::new(Vec::<argon2::Block>::new());
    memory.try_reserve_exact(count).map_err(|_| {
        Error::SecurityLimitExceeded("cannot allocate Argon2 working memory".into())
    })?;
    memory.resize(count, argon2::Block::new());
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = zeroize::Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into_with_memory(password, salt, &mut *key, &mut *memory)
        .map_err(|_| Error::InvalidKey)?;
    Ok(*key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_owned_argon_memory_preserves_password_key_bytes() {
        let password = b"synthetic compatibility password";
        let salt = [23u8; 16];
        let mut reference = zeroize::Zeroizing::new([0u8; 32]);
        let params = Params::new(64 * 1024, 3, 1, Some(32)).unwrap();
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(password, &salt, &mut *reference)
            .unwrap();
        let actual =
            zeroize::Zeroizing::new(derive_key_from_password_bytes(password, &salt).unwrap());
        assert_eq!(&*actual, &*reference);
        assert!(matches!(
            derive_key_from_password_bytes(password, &[0; 15]),
            Err(Error::InvalidInput(_))
        ));
    }
}
