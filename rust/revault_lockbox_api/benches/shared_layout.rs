//! Shared layout experiment, NOT a native lockbox writer or security protocol.
//! Same descriptor/table and read pipeline for every protection/codec mode.
//! Synthetic isolated frames: excludes TOC, allocation, recovery, padding,
//! variables/forms, full commit signing and external-storage revision handling.
use chacha20poly1305::{
    aead::{Aead, AeadInOut, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::ops::Range;
use zeroize::{Zeroize, Zeroizing};

type Result<T> = std::result::Result<T, &'static str>;
const MAX_FRAME: usize = 4 * 1024 * 1024;
const TAG: usize = 16;

#[derive(Clone)]
pub(crate) struct Descriptor {
    archive: [u8; 16],
    frame: u64,
    logical: usize,
    stored: usize,
    block: usize,
    compressed: bool,
    encrypted: bool,
    signed: bool,
    salt: [u8; 32],
    index_digest: [u8; 32],
    signature: Option<Signature>,
}

impl Descriptor {
    fn context(&self) -> Vec<u8> {
        let mut out = b"reVault shared-block EXPERIMENT 1\0".to_vec();
        out.extend_from_slice(&self.archive);
        for n in [
            self.frame,
            self.logical as u64,
            self.stored as u64,
            self.block as u64,
        ] {
            out.extend_from_slice(&n.to_le_bytes());
        }
        out.extend_from_slice(&[
            self.compressed as u8,
            self.encrypted as u8,
            self.signed as u8,
        ]);
        out.extend_from_slice(&self.salt);
        out
    }

    fn count(&self) -> Result<usize> {
        if self.logical > MAX_FRAME
            || self.stored > MAX_FRAME
            || ![16384, 65536, 262144].contains(&self.block)
            || (!self.compressed && self.stored != self.logical)
        {
            return Err("invalid bounded descriptor");
        }
        Ok(self.stored.div_ceil(self.block))
    }

    fn index_len(&self) -> Result<usize> {
        Ok(self.context().len() + self.count()? * 32 + if self.encrypted { TAG } else { 0 })
    }

    fn cipher(&self, key: Option<&[u8; 32]>) -> Result<Option<ChaCha20Poly1305>> {
        if !self.encrypted {
            return Ok(None);
        }
        let key = key.ok_or("missing key")?;
        let mut derived = Zeroizing::new([0u8; 32]);
        Hkdf::<Sha256>::new(Some(&self.salt), key)
            .expand(&self.context(), &mut *derived)
            .map_err(|_| "key derivation")?;
        Ok(Some(
            ChaCha20Poly1305::new_from_slice(&*derived).map_err(|_| "key")?,
        ))
    }

    fn aad(&self, index: bool, ordinal: usize) -> Vec<u8> {
        let mut out = self.context();
        out.push(index as u8);
        out.extend_from_slice(&(ordinal as u64).to_le_bytes());
        out
    }
}

fn nonce(index: bool, ordinal: usize) -> [u8; 12] {
    let mut out = [0; 12];
    out[0] = index as u8;
    out[4..].copy_from_slice(&(ordinal as u64).to_le_bytes());
    out
}

fn protect(
    cipher: &Option<ChaCha20Poly1305>,
    bytes: &[u8],
    aad: &[u8],
    nonce: [u8; 12],
) -> Result<Vec<u8>> {
    match cipher {
        None => Ok(bytes.to_vec()),
        Some(cipher) => cipher
            .encrypt(&Nonce::from(nonce), Payload { msg: bytes, aad })
            .map_err(|_| "encrypt"),
    }
}

fn unprotect(
    cipher: &Option<ChaCha20Poly1305>,
    bytes: &[u8],
    aad: &[u8],
    nonce: [u8; 12],
) -> Result<Vec<u8>> {
    match cipher {
        None => Ok(bytes.to_vec()),
        Some(cipher) => cipher
            .decrypt(&Nonce::from(nonce), Payload { msg: bytes, aad })
            .map_err(|_| "authentication"),
    }
}

pub(crate) fn encode(
    input: &[u8],
    block: usize,
    compressed: bool,
    key: Option<&[u8; 32]>,
    signer: Option<&SigningKey>,
) -> Result<(Descriptor, Vec<u8>)> {
    if input.len() > MAX_FRAME {
        return Err("input too large");
    }
    let candidate = compressed
        .then(|| Zeroizing::new(zstd_complete::encoding::compress_slice_c_level(input, 3)));
    let compressed = candidate
        .as_ref()
        .is_some_and(|bytes| bytes.len() < input.len());
    let stored = if compressed {
        candidate.as_ref().unwrap().as_slice()
    } else {
        input
    };
    let mut descriptor = Descriptor {
        archive: [17; 16],
        frame: 23,
        logical: input.len(),
        stored: stored.len(),
        block,
        compressed,
        encrypted: key.is_some(),
        signed: signer.is_some(),
        salt: [0; 32],
        index_digest: [0; 32],
        signature: None,
    };
    // Fresh salt even when retrying the same archive/frame identity. This is
    // experimental key separation, not an audited persistent nonce design.
    getrandom::fill(&mut descriptor.salt).map_err(|_| "random")?;
    descriptor.count()?;
    let cipher = descriptor.cipher(key)?;
    let mut index = Zeroizing::new(descriptor.context());
    let index_len = descriptor.index_len()?;
    let mut out = Zeroizing::new(Vec::with_capacity(
        index_len
            + stored.len()
            + if descriptor.encrypted {
                descriptor.count()? * TAG
            } else {
                0
            },
    ));
    out.resize(index_len, 0);
    for (ordinal, bytes) in stored.chunks(block).enumerate() {
        let start = out.len();
        out.extend_from_slice(bytes);
        if let Some(cipher) = &cipher {
            let tag = cipher
                .encrypt_inout_detached(
                    &Nonce::from(nonce(false, ordinal)),
                    &descriptor.aad(false, ordinal),
                    (&mut out[start..]).into(),
                )
                .map_err(|_| "encrypt")?;
            out.extend_from_slice(&tag);
        }
        // In encrypted mode only ciphertext commitments are persisted.
        index.extend_from_slice(&Sha256::digest(&out[start..]));
    }
    let encoded_index = protect(&cipher, &index, &descriptor.aad(true, 0), nonce(true, 0))?;
    out[..index_len].copy_from_slice(&encoded_index);
    descriptor.index_digest = Sha256::digest(&encoded_index).into();
    if let Some(signer) = signer {
        let mut message = descriptor.context();
        message.extend_from_slice(&descriptor.index_digest);
        descriptor.signature = Some(signer.sign(&message));
    }
    Ok((descriptor, std::mem::take(&mut *out)))
}

pub(crate) trait ByteSource {
    fn len(&self) -> Result<usize>;
    fn read(&self, range: Range<usize>) -> Result<Vec<u8>>;
    fn validate(&self) -> Result<()> {
        self.len().map(|_| ())
    }
}

impl ByteSource for [u8] {
    fn len(&self) -> Result<usize> {
        Ok(self.len())
    }
    fn read(&self, range: Range<usize>) -> Result<Vec<u8>> {
        self.get(range).map(<[u8]>::to_vec).ok_or("source range")
    }
}

impl ByteSource for Vec<u8> {
    fn len(&self) -> Result<usize> {
        Ok(self.len())
    }
    fn read(&self, range: Range<usize>) -> Result<Vec<u8>> {
        self.as_slice().read(range)
    }
}

/// Bounded extent in an immutable fixture file. Production must additionally
/// retain archive locks and enforce external-source revision/failure latching.
struct FileSource {
    file: std::sync::Mutex<std::fs::File>,
    start: u64,
    length: usize,
}

impl ByteSource for FileSource {
    fn len(&self) -> Result<usize> {
        let actual = self
            .file
            .lock()
            .map_err(|_| "lock")?
            .metadata()
            .map_err(|_| "stat")?
            .len();
        let end = self
            .start
            .checked_add(self.length as u64)
            .ok_or("extent overflow")?;
        if actual < end {
            return Err("truncated extent");
        }
        Ok(self.length)
    }
    fn read(&self, range: Range<usize>) -> Result<Vec<u8>> {
        use std::io::{Read, Seek, SeekFrom};
        if range.start > range.end || range.end > self.len()? {
            return Err("source range");
        }
        let offset = self
            .start
            .checked_add(range.start as u64)
            .ok_or("offset overflow")?;
        let mut bytes = Zeroizing::new(vec![0; range.len()]);
        let mut file = self.file.lock().map_err(|_| "lock")?;
        file.seek(SeekFrom::Start(offset)).map_err(|_| "seek")?;
        file.read_exact(&mut bytes).map_err(|_| "read")?;
        Ok(std::mem::take(&mut *bytes))
    }
}

pub(crate) struct Reader<'a, S: ByteSource + ?Sized> {
    descriptor: &'a Descriptor,
    source: &'a S,
    hashes: Vec<[u8; 32]>,
    cipher: Option<ChaCha20Poly1305>,
}

impl<'a, S: ByteSource + ?Sized> Reader<'a, S> {
    pub(crate) fn open(
        descriptor: &'a Descriptor,
        source: &'a S,
        key: Option<&[u8; 32]>,
        signer: Option<&VerifyingKey>,
    ) -> Result<Self> {
        let count = descriptor.count()?;
        let index_len = descriptor.index_len()?;
        let expected =
            index_len + descriptor.stored + if descriptor.encrypted { count * TAG } else { 0 };
        if source.len()? != expected {
            return Err("extent mismatch");
        }
        let encoded_index = Zeroizing::new(source.read(0..index_len)?);
        if encoded_index.len() != index_len {
            return Err("short index read");
        }
        if <[u8; 32]>::from(Sha256::digest(&encoded_index)) != descriptor.index_digest {
            return Err("index commitment");
        }
        if descriptor.signed {
            let mut message = descriptor.context();
            message.extend_from_slice(&descriptor.index_digest);
            signer
                .ok_or("missing signer")?
                .verify_strict(
                    &message,
                    descriptor.signature.as_ref().ok_or("missing signature")?,
                )
                .map_err(|_| "signature")?;
        } else if descriptor.signature.is_some() {
            return Err("unexpected signature");
        }
        let cipher = descriptor.cipher(key)?;
        let index = Zeroizing::new(unprotect(
            &cipher,
            &encoded_index,
            &descriptor.aad(true, 0),
            nonce(true, 0),
        )?);
        let context = descriptor.context();
        if index.len() != context.len() + count * 32 || !index.starts_with(&context) {
            return Err("index context");
        }
        let hashes = index[context.len()..]
            .chunks_exact(32)
            .map(|h| h.try_into().unwrap())
            .collect();
        let reader = Self {
            descriptor,
            source,
            hashes,
            cipher,
        };
        // Model the existing eager signed-content damage detection, rather
        // than quietly treating a valid signed table as a full content check.
        if descriptor.signed {
            let _checked = Zeroizing::new(reader.read(0..descriptor.logical)?);
        }
        Ok(reader)
    }

    pub(crate) fn read(&self, range: Range<usize>) -> Result<Vec<u8>> {
        // Cached index state and empty ranges cannot bypass source revocation.
        self.source.validate()?;
        let d = self.descriptor;
        if range.start > range.end || range.end > d.logical {
            return Err("range");
        }
        if range.is_empty() {
            return Ok(Vec::new());
        }
        let wanted = if d.compressed {
            0..d.stored
        } else {
            range.clone()
        };
        let first = wanted.start / d.block;
        let last = wanted.end.div_ceil(d.block);
        let tag_len = if d.encrypted { TAG } else { 0 };
        let stride = d.block + tag_len;
        let data_start = d.index_len()?;
        let extent_start = data_start + first * stride;
        let extent_end = data_start + (last * d.block).min(d.stored) + last * tag_len;
        let mut decoded = Zeroizing::new(self.source.read(extent_start..extent_end)?);
        if decoded.len() != extent_end - extent_start {
            return Err("short data read");
        }
        let mut plain_len = 0;
        for ordinal in first..last {
            let len = (d.stored - ordinal * d.block).min(d.block);
            let start = (ordinal - first) * stride;
            let block = &mut decoded[start..start + len + tag_len];
            if <[u8; 32]>::from(Sha256::digest(&*block)) != self.hashes[ordinal] {
                return Err("block commitment");
            }
            if let Some(cipher) = &self.cipher {
                let (message, tag_bytes) = block.split_at_mut(len);
                let tag = chacha20poly1305::Tag::try_from(&*tag_bytes).map_err(|_| "tag")?;
                cipher
                    .decrypt_inout_detached(
                        &Nonce::from(nonce(false, ordinal)),
                        &d.aad(false, ordinal),
                        message.into(),
                        &tag,
                    )
                    .map_err(|_| "authentication")?;
            }
            decoded.copy_within(start..start + len, plain_len);
            plain_len += len;
        }
        decoded[plain_len..].zeroize();
        decoded.truncate(plain_len);
        if d.compressed {
            let mut logical = Zeroizing::new(vec![0; d.logical]);
            let len = zstd_complete::decoding::FrameDecoder::new()
                .decode_all(&decoded, &mut logical)
                .map_err(|_| "decode")?;
            if len != d.logical {
                return Err("decoded length");
            }
            Ok(take_range(logical, range))
        } else {
            let start = range.start - first * d.block;
            Ok(take_range(decoded, start..start + range.len()))
        }
    }
}

fn take_range(mut bytes: Zeroizing<Vec<u8>>, range: Range<usize>) -> Vec<u8> {
    if range.start != 0 {
        bytes.copy_within(range.clone(), 0);
    }
    bytes[range.len()..].zeroize();
    bytes.truncate(range.len());
    std::mem::take(&mut *bytes)
}

// Also included as a module by the integration-test wrapper. Cargo sets
// cfg(test) for harness-free benches too, so main must remain available.
#[allow(dead_code)]
fn main() {
    use std::time::Instant;
    let size: usize = std::env::args()
        .nth(1)
        .expect("shared_layout SIZE")
        .parse()
        .unwrap();
    assert!(size <= MAX_FRAME, "prototype frame limit is 4 MiB");
    let root = std::env::args()
        .nth(2)
        .filter(|arg| arg != "-")
        .map(std::path::PathBuf::from);
    if let Some(root) = &root {
        assert!(root.is_dir(), "fixture directory must exist");
    }
    let storage_kind = if root.is_some() { "file" } else { "memory" };
    let corpus = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "pattern".to_owned());
    assert!(matches!(corpus.as_str(), "pattern" | "random"));
    let input: Vec<_> = (0..size)
        .map(|n| {
            if corpus == "pattern" {
                ((n * 13 + n / 251) % 251) as u8
            } else {
                let mut x = (n as u64).wrapping_add(0x9e3779b97f4a7c15);
                x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
                (x ^ (x >> 31)) as u8
            }
        })
        .collect();
    let signer = SigningKey::from_bytes(&[5; 32]);
    let public = signer.verifying_key();
    println!(
        "size,block,encrypted,signed,compressed,repeat,bytes,write_us,open_us,whole_us,range_us,compression_requested,storage,corpus"
    );
    for block in [16384, 65536, 262144] {
        for mode in 0..8 {
            let key = if mode & 1 != 0 { Some(&[71; 32]) } else { None };
            let signing = if mode & 2 != 0 { Some(&signer) } else { None };
            for repeat in 0..6 {
                let start = Instant::now();
                let (d, bytes) = encode(&input, block, mode & 4 != 0, key, signing).unwrap();
                let packet_len = bytes.len();
                let file_path = root
                    .as_ref()
                    .map(|root| root.join(format!("frame-{block}-{mode}-{repeat}.exp")));
                let source: Box<dyn ByteSource> = if let Some(path) = &file_path {
                    use std::io::Write;
                    let mut file = std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(path)
                        .unwrap();
                    file.write_all(&bytes).unwrap();
                    file.sync_all().unwrap();
                    Box::new(FileSource {
                        file: std::sync::Mutex::new(file),
                        start: 0,
                        length: packet_len,
                    })
                } else {
                    Box::new(bytes)
                };
                let write = start.elapsed().as_secs_f64() * 1e6;
                let start = Instant::now();
                let reader = Reader::open(&d, &*source, key, Some(&public)).unwrap();
                let open = start.elapsed().as_secs_f64() * 1e6;
                let start = Instant::now();
                let whole = Zeroizing::new(reader.read(0..size).unwrap());
                let whole_time = start.elapsed().as_secs_f64() * 1e6;
                assert_eq!(*whole, input);
                let range = size / 2..(size / 2 + 8192).min(size);
                let start = Instant::now();
                let part = Zeroizing::new(reader.read(range.clone()).unwrap());
                let range_time = start.elapsed().as_secs_f64() * 1e6;
                assert_eq!(*part, input[range]);
                if repeat > 0 {
                    println!("{size},{block},{},{},{},{repeat},{packet_len},{write:.3},{open:.3},{whole_time:.3},{range_time:.3},{},{storage_kind},{corpus}", d.encrypted, d.signed, d.compressed, mode & 4 != 0);
                }
                drop(reader);
                drop(source);
                // Only remove the exact fixture created with create_new above.
                if let Some(path) = file_path {
                    std::fs::remove_file(path).unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Harness-free benchmark builds omit #[test] functions but retain imports.
    #[allow(unused_imports)]
    use super::*;
    // Cargo retains this helper in harness-free bench builds but omits tests.
    #[allow(dead_code)]
    struct Observed<S> {
        inner: S,
        reads: std::cell::RefCell<Vec<Range<usize>>>,
        short_at: Option<usize>,
    }
    impl<S: ByteSource> ByteSource for Observed<S> {
        fn len(&self) -> Result<usize> {
            self.inner.len()
        }
        fn read(&self, range: Range<usize>) -> Result<Vec<u8>> {
            self.reads.borrow_mut().push(range.clone());
            let mut bytes = self.inner.read(range)?;
            if self.short_at == Some(self.reads.borrow().len()) {
                bytes.pop();
            }
            Ok(bytes)
        }
    }

    #[test]
    fn short_index_and_data_reads_fail_without_panics() {
        for short_at in [1, 2] {
            let (d, bytes) = encode(&vec![7; 40000], 16384, false, Some(&[71; 32]), None).unwrap();
            let source = Observed {
                inner: bytes,
                reads: Default::default(),
                short_at: Some(short_at),
            };
            let opened = Reader::open(&d, &source, Some(&[71; 32]), None);
            if short_at == 1 {
                assert!(opened.is_err());
            } else {
                assert!(opened.unwrap().read(17000..17100).is_err());
            }
        }
    }

    #[test]
    fn file_extent_reads_only_index_and_required_blocks_in_every_mode() {
        // Low-level frame fixture, not CLI E2E: no public CLI writes this
        // experimental encoding. Prefix/suffix model neighboring page bytes.
        use std::io::Write;
        let root = std::env::temp_dir().join(format!(
            "revault-shared-source-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let input: Vec<_> = (0..40000)
            .map(|n| ((n * 13 + n / 251) % 251) as u8)
            .collect();
        let signer = SigningKey::from_bytes(&[5; 32]);
        for mode in 0..8 {
            let key = if mode & 1 != 0 { Some(&[71; 32]) } else { None };
            let (d, bytes) = encode(
                &input,
                16384,
                mode & 4 != 0,
                key,
                if mode & 2 != 0 { Some(&signer) } else { None },
            )
            .unwrap();
            let path = root.join(format!("mode-{mode}.exp"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            file.write_all(&[0; 317]).unwrap();
            file.write_all(&bytes).unwrap();
            file.write_all(&[0; 29]).unwrap();
            file.sync_all().unwrap();
            drop(file);
            let source = Observed {
                inner: FileSource {
                    file: std::sync::Mutex::new(std::fs::File::open(&path).unwrap()),
                    start: 317,
                    length: bytes.len(),
                },
                reads: Default::default(),
                short_at: None,
            };
            let reader = Reader::open(&d, &source, key, Some(&signer.verifying_key())).unwrap();
            let index = d.index_len().unwrap();
            let mut expected = std::iter::once(0..index).collect::<Vec<_>>();
            if d.signed {
                expected.push(index..bytes.len());
            }
            assert_eq!(*source.reads.borrow(), expected);
            source.reads.borrow_mut().clear();
            assert_eq!(reader.read(17000..17100).unwrap(), input[17000..17100]);
            let stride = d.block + if d.encrypted { TAG } else { 0 };
            let expected = if d.compressed {
                index..bytes.len()
            } else {
                index + stride..index + stride * 2
            };
            assert_eq!(*source.reads.borrow(), vec![expected]);
            std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(317 + bytes.len() as u64 - 1)
                .unwrap();
            assert!(reader.read(17000..17100).is_err());
            drop(reader);
            drop(source);
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn associated_data_binds_block_identity_and_index_domain() {
        let (d, bytes) = encode(&vec![7; 40000], 16384, false, Some(&[71; 32]), None).unwrap();
        let cipher = d.cipher(Some(&[71; 32])).unwrap();
        let start = d.index_len().unwrap();
        let block = &bytes[start..start + 16384 + TAG];
        assert!(unprotect(&cipher, block, &d.aad(false, 1), nonce(false, 1)).is_err());
        assert!(unprotect(&cipher, block, &d.aad(true, 0), nonce(true, 0)).is_err());
        let mut bad = block.to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(unprotect(&cipher, &bad, &d.aad(false, 0), nonce(false, 0)).is_err());
        assert_eq!(
            unprotect(&cipher, block, &d.aad(false, 0), nonce(false, 0)).unwrap(),
            vec![7; 16384]
        );
    }

    #[test]
    fn untrusted_descriptor_and_signer_are_rejected() {
        let signer = SigningKey::from_bytes(&[5; 32]);
        let (d, bytes) =
            encode(&vec![7; 40000], 16384, true, Some(&[71; 32]), Some(&signer)).unwrap();
        assert!(Reader::open(&d, &bytes, Some(&[71; 32]), None).is_err());
        assert!(Reader::open(
            &d,
            &bytes,
            Some(&[71; 32]),
            Some(&SigningKey::from_bytes(&[6; 32]).verifying_key())
        )
        .is_err());
        for field in 0..5 {
            let mut bad = d.clone();
            match field {
                0 => bad.logical = usize::MAX,
                1 => bad.stored = usize::MAX,
                2 => bad.block = 0,
                3 => bad.archive[0] ^= 1,
                4 => bad.salt[0] ^= 1,
                _ => unreachable!(),
            }
            assert!(
                Reader::open(&bad, &bytes, Some(&[71; 32]), Some(&signer.verifying_key())).is_err()
            );
        }
    }
    #[test]
    fn all_modes_share_layout_and_roundtrip_ranges() {
        let signer = SigningKey::from_bytes(&[5; 32]);
        for size in [0, 1, 128, 16385, 65537] {
            let input: Vec<_> = (0..size).map(|n| (n % 251) as u8).collect();
            for mode in 0..8 {
                let key = if mode & 1 != 0 { Some(&[71; 32]) } else { None };
                let (d, bytes) = encode(
                    &input,
                    16384,
                    mode & 4 != 0,
                    key,
                    if mode & 2 != 0 { Some(&signer) } else { None },
                )
                .unwrap();
                let reader = Reader::open(&d, &bytes, key, Some(&signer.verifying_key())).unwrap();
                for range in [0..size, 0..0, size / 2..size] {
                    assert_eq!(reader.read(range.clone()).unwrap(), input[range]);
                }
                assert!(Reader::open(
                    &d,
                    &bytes[..bytes.len() - 1],
                    key,
                    Some(&signer.verifying_key())
                )
                .is_err());
                if key.is_some() {
                    assert!(Reader::open(
                        &d,
                        &bytes,
                        Some(&[72; 32]),
                        Some(&signer.verifying_key())
                    )
                    .is_err());
                }
            }
        }
    }
    #[test]
    fn ciphertext_and_signed_damage_fail_closed_and_retries_get_fresh_salts() {
        let signer = SigningKey::from_bytes(&[5; 32]);
        let input = vec![7; 40000];
        for mode in 0..4 {
            let key = if mode & 1 != 0 { Some(&[71; 32]) } else { None };
            let signing = if mode & 2 != 0 { Some(&signer) } else { None };
            let (d, bytes) = encode(&input, 16384, false, key, signing).unwrap();
            let (retry, _) = encode(&input, 16384, false, key, signing).unwrap();
            assert_ne!(d.salt, retry.salt);
            let mut bad = bytes.clone();
            *bad.last_mut().unwrap() ^= 1;
            let opened = Reader::open(&d, &bad, key, Some(&signer.verifying_key()));
            if d.signed {
                assert!(opened.is_err());
            } else {
                let reader = opened.unwrap();
                assert_eq!(reader.read(0..1).unwrap(), [7]);
                assert!(reader.read(39999..40000).is_err());
            }
            let mut altered = d.clone();
            altered.frame += 1;
            assert!(Reader::open(&altered, &bytes, key, Some(&signer.verifying_key())).is_err());
        }
    }
}
