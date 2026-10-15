use base64ct::{Base64, Base64UrlUnpadded, Encoding};
use revault_lockbox_api::{ContactKeyPair, ContactPublicKey, Error, Result, SecretVec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use zeroize::Zeroize;

use crate::{decode_hex, encode_hex};

const PRIVATE_LABEL: &str = "LOCKBOX PRIVATE KEY";
const PUBLIC_LABEL: &str = "LOCKBOX PUBLIC KEY";
/// Number of SHA-256 prefix bytes used for a contact public-key fingerprint.
pub const PUBLIC_KEY_FINGERPRINT_LEN: usize = 16;
/// Number of bytes encoded by the short, human-comparable fingerprint code.
pub const FINGERPRINT_CODE_96_LEN: usize = 12;
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const KTY: &str = "AKP";
const ALG: &str = "X25519-ML-KEM-768";
const CRV: &str = "X25519-ML-KEM-768";

/// Supported contact key serialization formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    /// Lockbox PEM envelope containing a Lockbox JWK payload.
    LockboxPem,

    /// Single JSON Web Key object.
    Jwk,

    /// JSON Web Key Set containing one key.
    Jwks,

    /// Raw key bytes encoded as hexadecimal text.
    RawHex,
}

impl KeyFormat {
    /// Parses a CLI/user-facing key format name.
    ///
    /// Accepted names include `lockbox`, `lockbox-pem`, `pem`, `jwk`,
    /// `jwks`, `raw`, `raw-hex`, and `hex`.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "lockbox" | "lockbox-pem" | "pem" => Ok(Self::LockboxPem),
            "jwk" => Ok(Self::Jwk),
            "jwks" => Ok(Self::Jwks),
            "raw" | "raw-hex" | "hex" => Ok(Self::RawHex),
            _ => Err(Error::InvalidInput(format!(
                "unsupported key format: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwkKey {
    kty: String,
    alg: String,
    crv: String,
    kid: String,
    x: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    d: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_ops: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<JwkKey>,
}

/// Exports a contact private key in the requested format.
///
/// Private-key output is returned as `SecretVec` so the serialized secret
/// material is zeroized on drop. `LockboxPem`, `Jwk`, and `Jwks` include both
/// public-key metadata and the private key record; `RawHex` contains only the
/// private key record bytes encoded as hexadecimal text.
pub fn export_private_key(keypair: &ContactKeyPair, format: KeyFormat) -> Result<SecretVec> {
    let public = keypair.public_key();
    let public_bytes = public.to_bytes();
    let public_x = Base64UrlUnpadded::encode_string(&public_bytes);
    let kid = fingerprint(&public_bytes);
    let private_record = keypair.private_key_record()?;
    match format {
        KeyFormat::RawHex => hex_encode_secure(private_record),
        KeyFormat::Jwk => private_jwk_secure(&kid, &public_x, private_record),
        KeyFormat::Jwks => {
            let jwk = private_jwk_secure(&kid, &public_x, private_record)?;
            jwks_secure(&jwk)
        }
        KeyFormat::LockboxPem => {
            let jwk = private_jwk_secure(&kid, &public_x, private_record)?;
            pem_secure(PRIVATE_LABEL, &jwk)
        }
    }
}

/// Imports a contact private key from PEM, JWK, JWKS-compatible JWK, or hex.
///
/// The input buffer is normalized in place and remains secret memory for the
/// duration of parsing.
pub fn import_private_key(mut bytes: SecretVec) -> Result<ContactKeyPair> {
    normalize_private_key_record(&mut bytes)?;
    ContactKeyPair::from_private_key_record(bytes)
}

/// Reads and imports a contact private key from a file.
///
/// File contents are loaded into `SecretVec` before parsing so private material
/// uses the same zeroizing path as `import_private_key`.
pub fn import_private_key_file(path: impl AsRef<Path>) -> Result<ContactKeyPair> {
    let mut file = fs::File::open(path.as_ref()).map_err(|err| Error::Io(err.to_string()))?;
    let len = usize::try_from(
        file.metadata()
            .map_err(|err| Error::Io(err.to_string()))?
            .len(),
    )
    .map_err(|_| {
        Error::SecurityLimitExceeded("private key file length exceeds addressable memory".into())
    })?;
    let mut bytes = SecretVec::new();
    bytes.resize_zeroed(len)?;
    bytes.with_mut_bytes(|buffer| {
        file.read_exact(buffer)
            .map_err(|err| Error::Io(err.to_string()))
    })??;
    import_private_key(bytes)
}

/// Returns the stable fingerprint for a contact public key.
pub fn public_key_fingerprint(key: &ContactPublicKey) -> Vec<u8> {
    fingerprint_bytes(&key.to_bytes())
}

/// Formats fingerprint bytes as lowercase space-separated hexadecimal pairs.
pub fn format_fingerprint_hex_pairs(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().saturating_mul(3).saturating_sub(1));
    for (index, byte) in bytes.iter().enumerate() {
        if index != 0 {
            out.push(' ');
        }
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Formats the first 96 bits of a fingerprint as lower-case Crockford Base32.
///
/// The result is grouped as five four-character groups for phone use. The
/// alphabet intentionally excludes i, l, o, and u.
pub fn format_fingerprint_crockford_96(bytes: &[u8]) -> String {
    assert!(
        bytes.len() >= FINGERPRINT_CODE_96_LEN,
        "fingerprint must contain at least 96 bits"
    );
    let mut compact = String::with_capacity(20);
    let mut buffer = 0u32;
    let mut bits = 0usize;
    for byte in &bytes[..FINGERPRINT_CODE_96_LEN] {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let value = ((buffer >> bits) & 0x1f) as usize;
            compact.push(CROCKFORD_ALPHABET[value] as char);
            buffer &= bit_mask(bits);
        }
    }
    if bits > 0 {
        let value = ((buffer << (5 - bits)) & 0x1f) as usize;
        compact.push(CROCKFORD_ALPHABET[value] as char);
    }
    group_fingerprint_code(&compact)
}

/// Returns a lower-case word reading for a Crockford fingerprint code.
pub fn format_fingerprint_crockford_96_reading(code: &str) -> String {
    code.split('-')
        .map(|group| {
            group
                .chars()
                .filter_map(crockford_spoken_word)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" - ")
}

/// Decodes a 96-bit lower/upper-case Crockford fingerprint code.
///
/// Whitespace, hyphens, and underscores are ignored. The standard Crockford
/// aliases i/l for 1 and o for 0 are accepted on input, but never printed.
pub fn decode_fingerprint_crockford_96(value: &str) -> Result<[u8; FINGERPRINT_CODE_96_LEN]> {
    let mut values = Vec::with_capacity(20);
    for ch in value.chars() {
        if ch.is_ascii_whitespace() || ch == '-' || ch == '_' {
            continue;
        }
        let value = crockford_decode_value(ch).ok_or_else(|| {
            Error::InvalidInput(format!(
                "fingerprint code contains invalid Crockford character: {ch}"
            ))
        })?;
        values.push(value);
    }
    if values.len() != 20 {
        return Err(Error::InvalidInput(
            "fingerprint code must contain 20 Crockford characters for 96 bits".to_string(),
        ));
    }

    let mut out = [0u8; FINGERPRINT_CODE_96_LEN];
    let mut out_index = 0usize;
    let mut buffer = 0u32;
    let mut bits = 0usize;
    for value in values {
        buffer = (buffer << 5) | u32::from(value);
        bits += 5;
        while bits >= 8 && out_index < out.len() {
            bits -= 8;
            out[out_index] = ((buffer >> bits) & 0xff) as u8;
            out_index += 1;
            buffer &= bit_mask(bits);
        }
    }
    if out_index != out.len() || bits != 4 || buffer != 0 {
        return Err(Error::InvalidInput(
            "fingerprint code contains non-canonical trailing bits".to_string(),
        ));
    }
    Ok(out)
}

/// Decodes a public-key fingerprint written as hex, with optional separators.
pub fn decode_fingerprint_hex(value: &str) -> Result<Vec<u8>> {
    let compact = value
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace() && *byte != b':' && *byte != b'-')
        .collect::<Vec<_>>();
    let compact = String::from_utf8(compact)
        .map_err(|_| Error::InvalidInput("fingerprint is not valid UTF-8".to_string()))?;
    let fingerprint = decode_hex(&compact)
        .map_err(|_| Error::InvalidInput("fingerprint contains invalid hex".to_string()))?;
    if fingerprint.len() != PUBLIC_KEY_FINGERPRINT_LEN {
        return Err(Error::InvalidInput(format!(
            "fingerprint must contain {PUBLIC_KEY_FINGERPRINT_LEN} two-digit hex groups; short PINs are too small to authenticate a public key"
        )));
    }
    Ok(fingerprint)
}

fn group_fingerprint_code(compact: &str) -> String {
    let mut out = String::with_capacity(compact.len() + compact.len() / 4);
    for (index, ch) in compact.chars().enumerate() {
        if index != 0 && index % 4 == 0 {
            out.push('-');
        }
        out.push(ch);
    }
    out
}

fn bit_mask(bits: usize) -> u32 {
    if bits == 0 {
        0
    } else {
        (1u32 << bits) - 1
    }
}

fn crockford_decode_value(ch: char) -> Option<u8> {
    match ch.to_ascii_lowercase() {
        '0' | 'o' => Some(0),
        '1' | 'i' | 'l' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' => Some(10),
        'b' => Some(11),
        'c' => Some(12),
        'd' => Some(13),
        'e' => Some(14),
        'f' => Some(15),
        'g' => Some(16),
        'h' => Some(17),
        'j' => Some(18),
        'k' => Some(19),
        'm' => Some(20),
        'n' => Some(21),
        'p' => Some(22),
        'q' => Some(23),
        'r' => Some(24),
        's' => Some(25),
        't' => Some(26),
        'v' => Some(27),
        'w' => Some(28),
        'x' => Some(29),
        'y' => Some(30),
        'z' => Some(31),
        _ => None,
    }
}

fn crockford_spoken_word(ch: char) -> Option<&'static str> {
    match ch {
        '0' => Some("zero"),
        '1' => Some("one"),
        '2' => Some("two"),
        '3' => Some("three"),
        '4' => Some("four"),
        '5' => Some("five"),
        '6' => Some("six"),
        '7' => Some("seven"),
        '8' => Some("eight"),
        '9' => Some("nine"),
        'a' => Some("alpha"),
        'b' => Some("bravo"),
        'c' => Some("charlie"),
        'd' => Some("delta"),
        'e' => Some("echo"),
        'f' => Some("foxtrot"),
        'g' => Some("golf"),
        'h' => Some("hotel"),
        'j' => Some("juliet"),
        'k' => Some("kilo"),
        'm' => Some("mike"),
        'n' => Some("november"),
        'p' => Some("papa"),
        'q' => Some("quebec"),
        'r' => Some("romeo"),
        's' => Some("sierra"),
        't' => Some("tango"),
        'v' => Some("victor"),
        'w' => Some("whiskey"),
        'x' => Some("xray"),
        'y' => Some("yankee"),
        'z' => Some("zulu"),
        _ => None,
    }
}

/// Exports a contact public key in the requested format.
pub fn export_public_key(key: &ContactPublicKey, format: KeyFormat) -> Result<Vec<u8>> {
    match format {
        KeyFormat::LockboxPem => pem(PUBLIC_LABEL, &public_jwk(key)?),
        KeyFormat::Jwk => {
            serde_json::to_vec_pretty(&public_jwk(key)?).map_err(|err| Error::Io(err.to_string()))
        }
        KeyFormat::Jwks => serde_json::to_vec_pretty(&Jwks {
            keys: vec![public_jwk(key)?],
        })
        .map_err(|err| Error::Io(err.to_string())),
        KeyFormat::RawHex => Ok(encode_hex(&key.to_bytes()).into_bytes()),
    }
}

/// Imports a contact public key from Lockbox PEM, JWK, JWKS, or raw hex.
pub fn import_public_key(bytes: &[u8]) -> Result<ContactPublicKey> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| Error::InvalidKeyMaterial("public key is not UTF-8 text".to_string()))?;
    if text_starts_with_pem_begin(text, PUBLIC_LABEL) {
        let (label, payload) = unpem(text)?;
        if label != PUBLIC_LABEL {
            return Err(Error::InvalidKeyMaterial(format!(
                "expected {PUBLIC_LABEL} PEM block, found {label}"
            )));
        }
        return public_from_jwk(&payload);
    }
    if text.trim_start().starts_with('{') {
        if let Ok(jwks) = serde_json::from_str::<Jwks>(text) {
            let key = jwks.keys.into_iter().next().ok_or_else(|| {
                Error::InvalidKeyMaterial("JWKS does not contain any keys".to_string())
            })?;
            return public_from_jwk(&key);
        }
        let key = serde_json::from_str::<JwkKey>(text).map_err(|_| {
            Error::InvalidKeyMaterial("public key JSON is not a supported JWK/JWKS".to_string())
        })?;
        return public_from_jwk(&key);
    }
    ContactPublicKey::from_bytes(&decode_hex(text.trim()).map_err(|_| {
        Error::InvalidKeyMaterial("public key is not valid hexadecimal bytes".to_string())
    })?)
}

fn private_jwk_secure(kid: &str, public_x: &str, private_record: SecretVec) -> Result<SecretVec> {
    let private_record = base64url_encode_secure(private_record)?;
    let mut out = SecretVec::new();
    out.try_extend_from_slice(
        br#"{"kty": "AKP", "alg": "X25519-ML-KEM-768", "crv": "X25519-ML-KEM-768", "kid": ""#,
    )?;
    out.try_extend_from_slice(kid.as_bytes())?;
    out.try_extend_from_slice(br#"", "x": ""#)?;
    out.try_extend_from_slice(public_x.as_bytes())?;
    out.try_extend_from_slice(br#"", "d": ""#)?;
    out.try_extend_from_secure(&private_record)?;
    out.try_extend_from_slice(br#"", "key_ops": ["unwrapKey", "deriveKey"]}"#)?;
    Ok(out)
}

fn jwks_secure(jwk: &SecretVec) -> Result<SecretVec> {
    let mut out = SecretVec::new();
    out.try_extend_from_slice(br#"{"keys": ["#)?;
    out.try_extend_from_secure(jwk)?;
    out.try_extend_from_slice(b"]}")?;
    Ok(out)
}

fn pem_secure(label: &str, payload: &SecretVec) -> Result<SecretVec> {
    let body = base64_encode_secure(payload.try_clone()?)?;
    let mut out = SecretVec::new();
    out.try_extend_from_slice(b"-----BEGIN ")?;
    out.try_extend_from_slice(label.as_bytes())?;
    out.try_extend_from_slice(b"-----\n")?;
    append_wrapped_base64(&mut out, &body)?;
    out.try_extend_from_slice(b"-----END ")?;
    out.try_extend_from_slice(label.as_bytes())?;
    out.try_extend_from_slice(b"-----\n")?;
    Ok(out)
}

fn append_wrapped_base64(out: &mut SecretVec, body: &SecretVec) -> Result<()> {
    let len = body.len();
    let mut offset = 0usize;
    while offset < len {
        let chunk = (len - offset).min(64);
        out.try_extend_secure_range(body, offset, chunk)?;
        out.try_push(b'\n')?;
        offset += chunk;
    }
    Ok(())
}

fn normalize_private_key_record(bytes: &mut SecretVec) -> Result<()> {
    bytes
        .with_mut_bytes(|bytes| {
            let (start, end) = trim_ascii_range(bytes);
            if starts_with_pem_begin(&bytes[start..end], PRIVATE_LABEL.as_bytes()) {
                let body_len = compact_pem_body(bytes, start, end)?;
                let decoded_len = Base64::decode_in_place(&mut bytes[..body_len])
                    .map_err(|_| {
                        Error::InvalidKeyMaterial(
                            "private PEM body is not valid base64".to_string(),
                        )
                    })?
                    .len();
                let (d_start, d_end) = find_json_string_value(&bytes[..decoded_len], b"d")?;
                bytes.copy_within(d_start..d_end, 0);
                let record_len = Base64UrlUnpadded::decode_in_place(&mut bytes[..d_end - d_start])
                    .map_err(|_| {
                        Error::InvalidKeyMaterial(
                            "private JWK d value is not valid base64url".to_string(),
                        )
                    })?
                    .len();
                return Ok(record_len);
            }
            if bytes[start..end].starts_with(b"{") {
                let (d_start, d_end) = find_json_string_value(&bytes[start..end], b"d")?;
                let d_start = start + d_start;
                let d_end = start + d_end;
                bytes.copy_within(d_start..d_end, 0);
                let record_len = Base64UrlUnpadded::decode_in_place(&mut bytes[..d_end - d_start])
                    .map_err(|_| {
                        Error::InvalidKeyMaterial(
                            "private JWK d value is not valid base64url".to_string(),
                        )
                    })?
                    .len();
                return Ok(record_len);
            }
            bytes.copy_within(start..end, 0);
            hex_decode_in_place(&mut bytes[..end - start])
        })?
        .and_then(|len| bytes.truncate(len).map_err(Into::into))
}

fn compact_pem_body(bytes: &mut [u8], start: usize, end: usize) -> Result<usize> {
    let mut line_start = start;
    let first_line_end = line_end(bytes, line_start, end);
    if !is_pem_boundary_line(
        &bytes[line_start..first_line_end],
        b"BEGIN",
        PRIVATE_LABEL.as_bytes(),
    ) {
        return Err(Error::InvalidKeyMaterial(
            "private PEM does not start with LOCKBOX PRIVATE KEY header".to_string(),
        ));
    }
    line_start = next_line_start(bytes, first_line_end, end);
    if line_start >= end {
        return Err(Error::InvalidKeyMaterial(
            "private PEM does not contain a body".to_string(),
        ));
    }

    let mut write = 0usize;
    let mut saw_end = false;
    while line_start < end {
        let line_end = line_end(bytes, line_start, end);
        if is_pem_boundary_line(
            &bytes[line_start..line_end],
            b"END",
            PRIVATE_LABEL.as_bytes(),
        ) {
            saw_end = true;
            break;
        }
        let mut read = line_start;
        while read < line_end {
            let byte = bytes[read];
            if !byte.is_ascii_whitespace() {
                bytes[write] = byte;
                write += 1;
            }
            read += 1;
        }
        line_start = next_line_start(bytes, line_end, end);
    }
    if !saw_end {
        return Err(Error::InvalidKeyMaterial(
            "private PEM input does not contain a matching END line".to_string(),
        ));
    }
    bytes[write..].zeroize();
    Ok(write)
}

fn starts_with_pem_begin(bytes: &[u8], label: &[u8]) -> bool {
    let end = line_end(bytes, 0, bytes.len());
    is_pem_boundary_line(&bytes[..end], b"BEGIN", label)
}

fn line_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut index = start;
    while index < end && bytes[index] != b'\n' && bytes[index] != b'\r' {
        index += 1;
    }
    index
}

fn next_line_start(bytes: &[u8], mut index: usize, end: usize) -> usize {
    while index < end && (bytes[index] == b'\n' || bytes[index] == b'\r') {
        index += 1;
    }
    index
}

fn is_pem_boundary_line(line: &[u8], keyword: &[u8], label: &[u8]) -> bool {
    let line = trim_matching_byte(trim_ascii(line), b'-');
    let Some(rest) = line.strip_prefix(keyword) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(b" ") else {
        return false;
    };
    rest == label
}

fn trim_matching_byte(mut bytes: &[u8], byte: u8) -> &[u8] {
    while bytes.first() == Some(&byte) {
        bytes = &bytes[1..];
    }
    while bytes.last() == Some(&byte) {
        bytes = &bytes[..bytes.len() - 1];
    }
    trim_ascii(bytes)
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

fn find_json_string_value(bytes: &[u8], key: &[u8]) -> Result<(usize, usize)> {
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'"' {
            index += 1;
            continue;
        }
        let key_start = index + 1;
        let Some(key_end) = bytes[key_start..].iter().position(|byte| *byte == b'"') else {
            return Err(Error::InvalidKeyMaterial(
                "JSON key name is unterminated".to_string(),
            ));
        };
        let key_end = key_start + key_end;
        index = key_end + 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b':' {
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if &bytes[key_start..key_end] == key {
            if index >= bytes.len() || bytes[index] != b'"' {
                return Err(Error::InvalidKeyMaterial(
                    "JSON key value is not a string".to_string(),
                ));
            }
            let value_start = index + 1;
            let Some(value_end) = bytes[value_start..].iter().position(|byte| *byte == b'"') else {
                return Err(Error::InvalidKeyMaterial(
                    "JSON string value is unterminated".to_string(),
                ));
            };
            return Ok((value_start, value_start + value_end));
        }
    }
    Err(Error::InvalidKeyMaterial(
        "private key JSON does not contain required d value".to_string(),
    ))
}

fn hex_encode_secure(mut bytes: SecretVec) -> Result<SecretVec> {
    let original_len = bytes.len();
    bytes.resize_zeroed(original_len * 2)?;
    bytes.with_mut_bytes(|bytes| {
        for index in (0..original_len).rev() {
            let byte = bytes[index];
            bytes[index * 2] = hex_digit(byte >> 4);
            bytes[index * 2 + 1] = hex_digit(byte & 0x0f);
        }
    })?;
    Ok(bytes)
}

fn base64url_encode_secure(bytes: SecretVec) -> Result<SecretVec> {
    base64_encode_with::<Base64UrlUnpadded>(bytes)
}

fn base64_encode_secure(bytes: SecretVec) -> Result<SecretVec> {
    base64_encode_with::<Base64>(bytes)
}

fn base64_encode_with<E: Encoding>(mut bytes: SecretVec) -> Result<SecretVec> {
    let original_len = bytes.len();
    let encoded_len = bytes.with_bytes(E::encoded_len)?;
    bytes.resize_zeroed(original_len + encoded_len)?;
    bytes.with_mut_bytes(|bytes| {
        bytes.copy_within(0..original_len, encoded_len);
        let (out, input) = bytes.split_at_mut(encoded_len);
        E::encode(&input[..original_len], out).map_err(|_| {
            Error::InvalidKeyMaterial("key bytes could not be base64 encoded".to_string())
        })?;
        input.zeroize();
        Ok::<_, Error>(())
    })??;
    bytes.truncate(encoded_len)?;
    Ok(bytes)
}

fn hex_decode_in_place(bytes: &mut [u8]) -> Result<usize> {
    if !bytes.len().is_multiple_of(2) {
        return Err(Error::InvalidKeyMaterial(
            "hex key has an odd number of digits".to_string(),
        ));
    }
    let out_len = bytes.len() / 2;
    for index in 0..out_len {
        let high = hex_value(bytes[index * 2])?;
        let low = hex_value(bytes[index * 2 + 1])?;
        bytes[index] = (high << 4) | low;
    }
    bytes[out_len..].zeroize();
    Ok(out_len)
}

fn hex_digit(value: u8) -> u8 {
    b"0123456789abcdef"[value as usize]
}

fn hex_value(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::InvalidKeyMaterial(format!(
            "hex key contains non-hex byte 0x{byte:02x}"
        ))),
    }
}

fn trim_ascii_range(bytes: &[u8]) -> (usize, usize) {
    let mut start = 0usize;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start, end)
}

fn public_jwk(key: &ContactPublicKey) -> Result<JwkKey> {
    let public_bytes = key.to_bytes();
    Ok(JwkKey {
        kty: KTY.to_string(),
        alg: ALG.to_string(),
        crv: CRV.to_string(),
        kid: fingerprint(&public_bytes),
        x: Base64UrlUnpadded::encode_string(&public_bytes),
        d: None,
        key_ops: Some(vec!["wrapKey".to_string()]),
    })
}

fn public_from_jwk(key: &JwkKey) -> Result<ContactPublicKey> {
    validate_jwk_header(key)?;
    let public = Base64UrlUnpadded::decode_vec(&key.x).map_err(|_| {
        Error::InvalidKeyMaterial("public JWK x value is not valid base64url".to_string())
    })?;
    ContactPublicKey::from_bytes(&public)
}

fn validate_jwk_header(key: &JwkKey) -> Result<()> {
    if key.kty == KTY && key.alg == ALG && key.crv == CRV {
        Ok(())
    } else {
        Err(Error::InvalidKeyMaterial(format!(
            "unsupported JWK header kty={}, alg={}, crv={}",
            key.kty, key.alg, key.crv
        )))
    }
}

fn pem(label: &str, payload: &JwkKey) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(payload).map_err(|err| Error::Io(err.to_string()))?;
    let body = Base64::encode_string(&json);
    let mut out = String::new();
    out.push_str(&format!("-----BEGIN {label}-----\n"));
    for chunk in body.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).map_err(|_| {
            Error::InvalidKeyMaterial("generated PEM contains invalid UTF-8".into())
        })?);
        out.push('\n');
    }
    out.push_str(&format!("-----END {label}-----\n"));
    Ok(out.into_bytes())
}

fn unpem(text: &str) -> Result<(String, JwkKey)> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let begin = lines.next().ok_or_else(|| {
        Error::InvalidKeyMaterial("PEM input does not contain a BEGIN line".to_string())
    })?;
    let label = parse_pem_text_boundary(begin, "BEGIN", None)
        .ok_or_else(|| Error::InvalidKeyMaterial("PEM BEGIN line is malformed".to_string()))?;
    let mut body = String::new();
    for line in lines {
        if parse_pem_text_boundary(line, "END", Some(&label)).is_some() {
            let json = Base64::decode_vec(&body).map_err(|_| {
                Error::InvalidKeyMaterial("PEM body is not valid base64".to_string())
            })?;
            let key = serde_json::from_slice(&json).map_err(|_| {
                Error::InvalidKeyMaterial("PEM body is not a supported JWK".to_string())
            })?;
            return Ok((label, key));
        }
        body.push_str(line);
    }
    Err(Error::InvalidKeyMaterial(
        "PEM input does not contain a matching END line".to_string(),
    ))
}

fn text_starts_with_pem_begin(text: &str, label: &str) -> bool {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| parse_pem_text_boundary(line, "BEGIN", Some(label)))
        .is_some()
}

fn parse_pem_text_boundary(
    line: &str,
    keyword: &str,
    expected_label: Option<&str>,
) -> Option<String> {
    let core = line.trim().trim_matches('-').trim();
    let label = core.strip_prefix(keyword)?.strip_prefix(' ')?;
    if label.is_empty() {
        return None;
    }
    if let Some(expected_label) = expected_label {
        if label != expected_label {
            return None;
        }
    }
    Some(label.to_string())
}

fn fingerprint(public_key: &[u8]) -> String {
    encode_hex(&fingerprint_bytes(public_key))
}

fn fingerprint_bytes(public_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"lockbox-key-fingerprint-v1");
    hasher.update(public_key);
    hasher.finalize()[..PUBLIC_KEY_FINGERPRINT_LEN].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_pem_round_trips_private_and_public_keys() {
        let keypair = ContactKeyPair::generate().unwrap();
        let private = export_private_key(&keypair, KeyFormat::LockboxPem).unwrap();
        let loaded = import_private_key(private).unwrap();
        assert_eq!(
            loaded.private_key_record().unwrap(),
            keypair.private_key_record().unwrap()
        );

        let public = export_public_key(&keypair.public_key(), KeyFormat::LockboxPem).unwrap();
        let loaded_public = import_public_key(&public).unwrap();
        assert_eq!(loaded_public.to_bytes(), keypair.public_key().to_bytes());
    }

    #[test]
    fn pem_import_tolerates_shortened_boundary_dashes() {
        let keypair = ContactKeyPair::generate().unwrap();

        let private = export_private_key(&keypair, KeyFormat::LockboxPem).unwrap();
        let private = private
            .with_bytes(|bytes| String::from_utf8(bytes.to_vec()).unwrap())
            .unwrap()
            .replace(
                "-----BEGIN LOCKBOX PRIVATE KEY-----",
                "----BEGIN LOCKBOX PRIVATE KEY----",
            )
            .replace(
                "-----END LOCKBOX PRIVATE KEY-----",
                "----END LOCKBOX PRIVATE KEY----",
            );
        let loaded =
            import_private_key(SecretVec::try_from_slice(private.as_bytes()).unwrap()).unwrap();
        assert_eq!(
            loaded.private_key_record().unwrap(),
            keypair.private_key_record().unwrap()
        );

        let public = export_public_key(&keypair.public_key(), KeyFormat::LockboxPem).unwrap();
        let public = String::from_utf8(public)
            .unwrap()
            .replace(
                "-----BEGIN LOCKBOX PUBLIC KEY-----",
                "----BEGIN LOCKBOX PUBLIC KEY----",
            )
            .replace(
                "-----END LOCKBOX PUBLIC KEY-----",
                "----END LOCKBOX PUBLIC KEY----",
            );
        let loaded_public = import_public_key(public.as_bytes()).unwrap();
        assert_eq!(loaded_public.to_bytes(), keypair.public_key().to_bytes());
    }

    #[test]
    fn jwk_and_jwks_round_trip() {
        let keypair = ContactKeyPair::generate().unwrap();
        let jwk = export_private_key(&keypair, KeyFormat::Jwk).unwrap();
        let public_jwk = export_public_key(&keypair.public_key(), KeyFormat::Jwk).unwrap();
        let public_jwk_text = std::str::from_utf8(&public_jwk).unwrap();
        assert!(public_jwk_text.contains(&format!(
            r#""kid": "{}""#,
            encode_hex(&public_key_fingerprint(&keypair.public_key()))
        )));
        assert_eq!(
            import_private_key(jwk)
                .unwrap()
                .private_key_record()
                .unwrap(),
            keypair.private_key_record().unwrap()
        );
        let jwks = export_public_key(&keypair.public_key(), KeyFormat::Jwks).unwrap();
        assert_eq!(
            import_public_key(&jwks).unwrap().to_bytes(),
            keypair.public_key().to_bytes()
        );
    }

    #[test]
    fn raw_hex_remains_importable() {
        let keypair = ContactKeyPair::generate().unwrap();
        let raw = export_private_key(&keypair, KeyFormat::RawHex).unwrap();
        assert_eq!(
            import_private_key(raw)
                .unwrap()
                .private_key_record()
                .unwrap(),
            keypair.private_key_record().unwrap()
        );
    }
}
