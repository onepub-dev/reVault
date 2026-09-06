//! Browser Native Messaging uses native-endian u32 framing around UTF-8 JSON.
use crate::{Error, Message, MAX_PROTO_BYTES};
use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Tighter than either browser's limit; checked before allocation.
pub const MAX_JSON_BYTES: usize = 24 * 1024;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct JsonEnvelope {
    payload: String,
}

/// Read one bounded frame. EOF is clean only between frames.
pub fn read_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, Error> {
    let mut header = [0; 4];
    match reader.read(&mut header[..1]) {
        Ok(0) => return Ok(None),
        Ok(_) => (),
        Err(_) => return Err(Error::InvalidRequest),
    }
    reader
        .read_exact(&mut header[1..])
        .map_err(|_| Error::InvalidRequest)?;
    let len = u32::from_ne_bytes(header) as usize;
    if len == 0 || len > MAX_JSON_BYTES {
        return Err(Error::InvalidRequest);
    }
    let mut json = vec![0; len];
    reader
        .read_exact(&mut json)
        .map_err(|_| Error::InvalidRequest)?;
    decode_json(&json).map(Some)
}
/// Decode only the documented JSON envelope and bounded base64 payload.
pub fn decode_json(json: &[u8]) -> Result<Vec<u8>, Error> {
    if json.len() > MAX_JSON_BYTES {
        return Err(Error::InvalidRequest);
    }
    let envelope: JsonEnvelope = serde_json::from_slice(json).map_err(|_| Error::InvalidRequest)?;
    let bytes = Base64::decode_vec(&envelope.payload).map_err(|_| Error::InvalidRequest)?;
    if bytes.is_empty() || bytes.len() > MAX_PROTO_BYTES {
        return Err(Error::InvalidRequest);
    }
    Ok(bytes)
}
/// Write a ciphertext-only protobuf response as a Native Messaging frame.
pub fn write_frame(writer: &mut impl Write, message: &impl Message) -> Result<(), Error> {
    let bytes = message.encode_to_vec();
    if bytes.len() > MAX_PROTO_BYTES {
        return Err(Error::InvalidRequest);
    }
    let json = serde_json::to_vec(&JsonEnvelope {
        payload: Base64::encode_string(&bytes),
    })
    .map_err(|_| Error::Internal)?;
    writer
        .write_all(&(json.len() as u32).to_ne_bytes())
        .map_err(|_| Error::Internal)?;
    writer.write_all(&json).map_err(|_| Error::Internal)?;
    writer.flush().map_err(|_| Error::Internal)
}
