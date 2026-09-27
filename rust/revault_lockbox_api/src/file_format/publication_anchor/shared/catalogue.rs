//! Bounded private control-record envelope, not a typed object catalogue. Uses
//! existing compression/AEAD with a separate derived-key domain. Caller must first
//! authenticate the publication and verify its stored-byte commitment.
use super::*;
use crate::compression::{encode_with_compression, COMPRESSION_NONE, COMPRESSION_ZSTD};
use crate::crypto::{open_with_nonce, seal_with_random_nonce};
use crate::page_buffer::ZeroizingBytes;
use sha2::Sha256;
use zeroize::Zeroizing;
use zstd_complete::decoding::StaticDecoderWorkspace;
const HEADER: usize = 44;
const BODY_HEADER: usize = 12;
const MAX_PLAIN: usize = 65536;
const MAGIC: &[u8; 8] = b"RV4CAT01";
pub(super) struct Codec {
    archive: LockboxId,
    mode: FormatMode,
    key: Option<Zeroizing<[u8; 32]>>,
}
impl Codec {
    pub(super) fn new(archive: LockboxId, mode: FormatMode, key: Option<&[u8]>) -> Result<Self> {
        FormatMode::parse(mode.0)?;
        if mode.plaintext() != key.is_none() || key.is_some_and(|k| k.len() != 32) {
            return Err(Error::InvalidKey);
        }
        let key = key.map(|key| {
            let mut derived = Zeroizing::new([0; 32]);
            hkdf::Hkdf::<Sha256>::new(Some(archive.as_bytes()), key)
                .expand(b"revault-shared-control-catalogue-v1\0", &mut *derived)
                .expect("fixed key size");
            derived
        });
        Ok(Self { archive, mode, key })
    }
    pub(super) fn encode(&self, plain: &[u8]) -> Result<ZeroizingBytes> {
        if plain.len() > MAX_PLAIN {
            return Err(Error::SecurityLimitExceeded(
                "inline catalogue decoded limit".into(),
            ));
        }
        let (codec, encoded) = encode_with_compression(plain, self.mode.options().compression);
        let encoded = ZeroizingBytes::new(encoded);
        let body_len = PRIVATE_BYTES - HEADER - if self.key.is_some() { 16 } else { 0 };
        if BODY_HEADER + encoded.len() > body_len {
            return Err(Error::SecurityLimitExceeded(
                "catalogue needs authenticated overflow".into(),
            ));
        }
        let mut body = ZeroizingBytes::new(Vec::with_capacity(body_len));
        body.extend_from_slice(&(plain.len() as u32).to_le_bytes());
        body.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        body.push(codec);
        body.extend_from_slice(&[0; 3]);
        body.extend_from_slice(&encoded);
        body.resize(body_len, 0);
        let mut out = ZeroizingBytes::new(Vec::with_capacity(PRIVATE_BYTES));
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&self.mode.0.to_le_bytes());
        out.extend_from_slice(self.archive.as_bytes());
        if let Some(key) = &self.key {
            let (nonce, encrypted) = seal_with_random_nonce(&body, key.as_slice(), &out)?;
            out.extend_from_slice(&nonce);
            out.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());
            out.extend_from_slice(&encrypted);
        } else {
            out.extend_from_slice(&[0; 12]);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&body);
        }
        debug_assert_eq!(out.len(), PRIVATE_BYTES);
        Ok(out)
    }
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<ZeroizingBytes> {
        if bytes.len() != PRIVATE_BYTES
            || &bytes[..8] != MAGIC
            || bytes[8..10] != 1u16.to_le_bytes()
            || bytes[10..12] != self.mode.0.to_le_bytes()
            || bytes[12..28] != *self.archive.as_bytes()
            || u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize
                != PRIVATE_BYTES - HEADER
        {
            return Err(Error::CorruptRecord);
        }
        let body = if let Some(key) = &self.key {
            ZeroizingBytes::new(open_with_nonce(
                &bytes[HEADER..],
                key.as_slice(),
                &bytes[28..40],
                &bytes[..28],
            )?)
        } else {
            if bytes[28..40] != [0; 12] {
                return Err(Error::CorruptRecord);
            }
            ZeroizingBytes::new(bytes[HEADER..].to_vec())
        };
        let decoded_len = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
        let stored_len = u32::from_le_bytes(body[4..8].try_into().unwrap()) as usize;
        if decoded_len > MAX_PLAIN || stored_len > body.len() - BODY_HEADER || body[9..12] != [0; 3]
        {
            return Err(Error::CorruptRecord);
        }
        let stored = &body[BODY_HEADER..BODY_HEADER + stored_len];
        if body[BODY_HEADER + stored_len..].iter().any(|b| *b != 0) {
            return Err(Error::CorruptRecord);
        }
        match body[8] {
            COMPRESSION_NONE if decoded_len == stored.len() => {
                Ok(ZeroizingBytes::new(stored.to_vec()))
            }
            COMPRESSION_ZSTD if self.mode.options().compression != crate::Compression::None => {
                let required = StaticDecoderWorkspace::required_size(MAX_PLAIN, 0)
                    .map_err(|_| Error::CorruptRecord)?;
                let mut scratch = ZeroizingBytes::new(vec![0; required]);
                let mut output = ZeroizingBytes::new(vec![0; decoded_len]);
                let mut decoder = StaticDecoderWorkspace::new(&mut scratch, MAX_PLAIN, 0)
                    .map_err(|_| Error::CorruptRecord)?;
                if decoder
                    .decode_into(stored, &mut output)
                    .map_err(|_| Error::CorruptRecord)?
                    != decoded_len
                {
                    return Err(Error::CorruptRecord);
                }
                Ok(output)
            }
            _ => Err(Error::CorruptRecord),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
    #[test]
    fn fixed_private_envelope_round_trips_all_modes_and_binds_context() {
        let archive = LockboxId::from_bytes([61; 16]);
        let key = [62; 32];
        for encrypted in [false, true] {
            for signed in [false, true] {
                for compressed in [false, true] {
                    for padded in [false, true] {
                        let mode = FormatMode::new(LockboxFormatOptions {
                            encryption: if encrypted {
                                EncryptionMode::ChaCha20Poly1305
                            } else {
                                EncryptionMode::None
                            },
                            signing: if signed {
                                SigningMode::Owner
                            } else {
                                SigningMode::None
                            },
                            compression: if compressed {
                                Compression::default()
                            } else {
                                Compression::None
                            },
                            size_padding: if padded {
                                SizePadding::Default
                            } else {
                                SizePadding::None
                            },
                        });
                        let codec =
                            Codec::new(archive, mode, encrypted.then_some(key.as_slice())).unwrap();
                        for data in [
                            Vec::new(),
                            b"private catalogue with synthetic names".to_vec(),
                            vec![73; if compressed { MAX_PLAIN } else { 40000 }],
                        ] {
                            let encoded = codec.encode(&data).unwrap();
                            assert_eq!(encoded.len(), PRIVATE_BYTES);
                            assert_eq!(codec.decode(&encoded).unwrap().as_slice(), data);
                            assert!(Codec::new(
                                LockboxId::from_bytes([60; 16]),
                                mode,
                                encrypted.then_some(key.as_slice())
                            )
                            .unwrap()
                            .decode(&encoded)
                            .is_err());
                            let mut corrupt = encoded.clone();
                            corrupt[10] ^= 1;
                            assert!(codec.decode(&corrupt).is_err());
                            if encrypted {
                                assert!(Codec::new(archive, mode, Some(&[63; 32]))
                                    .unwrap()
                                    .decode(&encoded)
                                    .is_err());
                                let mut corrupt = encoded.clone();
                                corrupt[PRIVATE_BYTES - 1] ^= 1;
                                assert!(codec.decode(&corrupt).is_err());
                                assert_ne!(
                                    encoded.as_slice(),
                                    codec.encode(&data).unwrap().as_slice()
                                );
                            }
                        }
                        assert!(codec.encode(&vec![0; MAX_PLAIN + 1]).is_err());
                    }
                }
            }
        }
    }
    #[test]
    fn private_envelope_checks_lengths_padding_codec_and_overflow_before_decode() {
        let mode = FormatMode::new(LockboxFormatOptions {
            encryption: EncryptionMode::None,
            signing: SigningMode::None,
            compression: Compression::None,
            size_padding: SizePadding::Default,
        });
        let codec = Codec::new(LockboxId::from_bytes([64; 16]), mode, None).unwrap();
        assert!(codec.encode(&vec![1; PRIVATE_BYTES]).is_err());
        let encoded = codec.encode(b"bounded catalogue").unwrap();
        for field in [HEADER, HEADER + 4] {
            let mut corrupt = encoded.clone();
            corrupt[field..field + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(codec.decode(&corrupt).is_err());
        }
        for field in [HEADER + 8, HEADER + 9, PRIVATE_BYTES - 1, 28, 40] {
            let mut corrupt = encoded.clone();
            corrupt[field] = 255;
            assert!(codec.decode(&corrupt).is_err());
        }
        assert!(codec.decode(&encoded[..PRIVATE_BYTES - 1]).is_err());
        assert!(Codec::new(LockboxId::from_bytes([64; 16]), mode, Some(&[0; 32])).is_err());
    }
}
