//! RFC 9180: DHKEM(X25519, HKDF-SHA256), HKDF-SHA256, `ChaCha20Poly1305`.
//! Payload encryption/decryption happens in the existing protected allocator.
use crate::{Error, Message, UnlockEnvelope, UnlockRequest, HPKE_INFO};
use hpke::{
    aead::{AeadTag, ChaCha20Poly1305},
    kdf::HkdfSha256,
    kem::X25519HkdfSha256,
    Deserializable, Kem, OpModeR, OpModeS, Serializable,
};
use rand_core::{OsRng, TryRngCore};
use revault_lockbox_api::{Lockbox, LockboxOpen, ReadOnly, SecretVec};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

type K = X25519HkdfSha256;
/// A per-process recipient key. Never serialize or persist it.
pub struct RecipientKey {
    private: SecretVec,
    public: Vec<u8>,
}
impl RecipientKey {
    /// Create a fresh key for this server boot.
    ///
    /// # Errors
    /// Returns `Internal` if OS randomness or protected allocation is unavailable.
    pub fn generate() -> Result<Self, Error> {
        let mut seed = Zeroizing::new([0; 32]);
        OsRng
            .try_fill_bytes(&mut *seed)
            .map_err(|_| Error::Internal)?;
        let (private, public) = K::derive_keypair(&*seed);
        seed.zeroize();
        let mut encoded = private.to_bytes();
        let protected = SecretVec::try_from_slice(&encoded).map_err(|_| Error::Internal);
        encoded.zeroize();
        Ok(Self {
            private: protected?,
            public: public.to_bytes().to_vec(),
        })
    }
    /// Public encryption key to include in the signed server request.
    pub fn public_key(&self) -> &[u8] {
        &self.public
    }
    /// Decrypt only an envelope matching a server-held request byte for byte.
    /// The caller must atomically consume the authenticated session's challenge
    /// before calling this function, and must retain neither key nor agent cache
    /// across server restarts or explicit locks.
    ///
    /// # Errors
    /// Rejects expired, mismatched, or unauthentic envelopes. Protected allocation
    /// failures return `Internal`; no plaintext is returned on error.
    pub fn open(
        &self,
        envelope: &UnlockEnvelope,
        expected: &UnlockRequest,
        now: u64,
    ) -> Result<UnlockMaterial, Error> {
        crate::validation::validate_request(expected, now)?;
        if envelope.binding.as_ref() != Some(expected)
            || expected.recipient_public_key != self.public
            || envelope.ciphertext.len() != 32
        {
            return Err(Error::InvalidRequest);
        }
        let enc = <K as Kem>::EncappedKey::from_bytes(&envelope.encapsulated_key)
            .map_err(|_| Error::InvalidRequest)?;
        let tag = AeadTag::<ChaCha20Poly1305>::from_bytes(&envelope.tag)
            .map_err(|_| Error::InvalidRequest)?;
        let mut key =
            SecretVec::try_from_slice(&envelope.ciphertext).map_err(|_| Error::Internal)?;
        // The allocator forbids mutation during a read scope. The library private
        // key is a short-lived, zeroize-on-drop X25519 value at the crypto boundary.
        let private = self
            .private
            .with_bytes(<K as Kem>::PrivateKey::from_bytes)
            .map_err(|_| Error::Internal)?
            .map_err(|_| Error::Internal)?;
        key.with_mut_bytes(|ciphertext| {
            hpke::single_shot_open_in_place_detached::<ChaCha20Poly1305, HkdfSha256, K>(
                &OpModeR::Base,
                &private,
                &enc,
                HPKE_INFO,
                ciphertext,
                &expected.encode_to_vec(),
                &tag,
            )
        })
        .map_err(|_| Error::Internal)?
        .map_err(|_| Error::InvalidRequest)?;
        Ok(UnlockMaterial {
            key,
            lockbox_id: expected.lockbox_id.clone(),
        })
    }
}
/// Minimum unlock material held in protected memory. No plaintext export API.
pub struct UnlockMaterial {
    key: SecretVec,
    lockbox_id: String,
}
impl UnlockMaterial {
    /// Import into a read-only lockbox handle without populating a Session Agent.
    ///
    /// # Errors
    /// Returns `LockboxUnavailable` for an inaccessible file, wrong key or ID.
    pub fn open_lockbox(self, path: impl AsRef<Path>) -> Result<Lockbox<ReadOnly>, Error> {
        let lockbox = Lockbox::open(path.as_ref(), LockboxOpen::ContentKey(self.key))
            .map_err(|_| Error::LockboxUnavailable)?;
        if lockbox.lockbox_id().to_string() != self.lockbox_id {
            return Err(Error::LockboxUnavailable);
        }
        Ok(lockbox)
    }
}
/// Seal a content key inside the agent. Only ciphertext leaves protected memory.
///
/// # Errors
/// Rejects invalid key lengths or HPKE recipients; protected-memory failures
/// return `Internal`. Entropy failure in HPKE aborts rather than releasing a key.
pub fn seal(key: &SecretVec, binding: &UnlockRequest) -> Result<UnlockEnvelope, Error> {
    if key.len() != 32 {
        return Err(Error::LockboxUnavailable);
    }
    let public = <K as Kem>::PublicKey::from_bytes(&binding.recipient_public_key)
        .map_err(|_| Error::InvalidRequest)?;
    let mut buffer = key.try_clone().map_err(|_| Error::Internal)?;
    let (enc, tag) = buffer
        .with_mut_bytes(|bytes| {
            hpke::single_shot_seal_in_place_detached::<ChaCha20Poly1305, HkdfSha256, K, _>(
                &OpModeS::Base,
                &public,
                HPKE_INFO,
                bytes,
                &binding.encode_to_vec(),
                &mut OsRng.unwrap_err(),
            )
        })
        .map_err(|_| Error::Internal)?
        .map_err(|_| Error::InvalidRequest)?;
    let ciphertext = buffer
        .with_bytes(<[u8]>::to_vec)
        .map_err(|_| Error::Internal)?;
    Ok(UnlockEnvelope {
        binding: Some(binding.clone()),
        encapsulated_key: enc.to_bytes().to_vec(),
        ciphertext,
        tag: tag.to_bytes().to_vec(),
    })
}
