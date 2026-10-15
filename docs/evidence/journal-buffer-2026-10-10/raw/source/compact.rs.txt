//! Shared-control preparation representation only. This codec/selector does not
//! grant reservation or cleanup authority. The allocator must validate a selected
//! record against authenticated ownership before acting, and must durably own an
//! overflow append before writing it. Existing 2,048-reservation capacity remains.
use super::*;
use crate::file_format::publication_anchor::{RootRef, FAILURE_REGION};
const STUB_BYTES: usize = 4096;
const STUB_OFFSET: u64 = 8192;
const STUB_MAGIC: &[u8; 8] = b"RV4PST01";
const STUB_CHECKSUM: usize = STUB_BYTES - 32;
struct Encoded {
    stub: Vec<u8>,
    overflow: Option<Vec<u8>>,
}
enum Contents {
    Inline(Record),
    Overflow(RootRef),
}
struct Stub {
    sequence: u64,
    previous: [u8; 32],
    base: [u8; 32],
    contents: Contents,
    digest: [u8; 32],
}
fn validate_reference(reference: RootRef) -> Result<()> {
    if reference.len != SLOT_BYTES as u64
        || reference.primary < REGION_LEN as u64
        || reference.mirror < REGION_LEN as u64
        || reference.primary % FAILURE_REGION != 0
        || reference.mirror % FAILURE_REGION != 0
        || reference.primary == reference.mirror
        || reference.primary.checked_add(reference.len).is_none()
        || reference.mirror.checked_add(reference.len).is_none()
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}
fn encode(context: &Context, record: &Record, locations: Option<(u64, u64)>) -> Result<Encoded> {
    let body_len = STUB_CHECKSUM - HEADER - if context.key.is_some() { 16 } else { 0 };
    // Existing validation runs before reservation count arithmetic/serialization.
    if record.reservations.len() > MAX_RESERVATIONS {
        return Err(Error::CorruptRecord);
    }
    let inline = 117 + record.reservations.len() * 25 < body_len;
    let mut body = Zeroizing::new(Vec::with_capacity(body_len));
    let overflow = if inline {
        if locations.is_some() {
            return Err(Error::InvalidInput(
                "inline record must not allocate overflow".into(),
            ));
        }
        body.push(0);
        body.extend_from_slice(&encode_body(record, body_len - 1)?);
        None
    } else {
        let (primary, mirror) = locations.ok_or_else(|| {
            Error::SecurityLimitExceeded("preparation record requires mirrored overflow".into())
        })?;
        let bytes = context.encode(record)?;
        let reference = RootRef {
            primary,
            mirror,
            len: bytes.len() as u64,
            digest: strong_checksum(&bytes),
        };
        validate_reference(reference)?;
        body.push(1);
        body.extend_from_slice(&record.sequence.to_le_bytes());
        body.extend_from_slice(&record.previous);
        body.extend_from_slice(&record.base);
        body.extend_from_slice(&reference.primary.to_le_bytes());
        body.extend_from_slice(&reference.mirror.to_le_bytes());
        body.extend_from_slice(&reference.len.to_le_bytes());
        body.extend_from_slice(&reference.digest);
        body.resize(body_len, 0);
        Some(bytes)
    };
    let mut stub = Vec::with_capacity(STUB_BYTES);
    stub.extend_from_slice(STUB_MAGIC);
    stub.extend_from_slice(&1u16.to_le_bytes());
    stub.extend_from_slice(&context.mode.0.to_le_bytes());
    stub.extend_from_slice(context.archive.as_bytes());
    stub.extend_from_slice(&(STUB_BYTES as u32).to_le_bytes());
    if let Some(key) = &context.key {
        let (nonce, encrypted) = seal_with_random_nonce(&body, key.as_slice(), &stub)?;
        stub.extend_from_slice(&nonce);
        stub.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());
        stub.extend_from_slice(&encrypted);
    } else {
        stub.extend_from_slice(&[0; 12]);
        stub.extend_from_slice(&(body.len() as u32).to_le_bytes());
        stub.extend_from_slice(&body);
    }
    debug_assert_eq!(stub.len(), STUB_CHECKSUM);
    let checksum = strong_checksum(&stub);
    stub.extend_from_slice(&checksum);
    Ok(Encoded { stub, overflow })
}
fn decode(context: &Context, bytes: &[u8]) -> Result<Stub> {
    if bytes.len() != STUB_BYTES
        || &bytes[..8] != STUB_MAGIC
        || bytes[8..10] != 1u16.to_le_bytes()
        || bytes[10..12] != context.mode.0.to_le_bytes()
        || bytes[12..28] != *context.archive.as_bytes()
        || bytes[28..32] != (STUB_BYTES as u32).to_le_bytes()
        || bytes[44..48] != ((STUB_CHECKSUM - HEADER) as u32).to_le_bytes()
        || strong_checksum(&bytes[..STUB_CHECKSUM]) != bytes[STUB_CHECKSUM..]
    {
        return Err(Error::CorruptRecord);
    }
    let body = if let Some(key) = &context.key {
        crate::page_buffer::ZeroizingBytes::new(open_with_nonce(
            &bytes[HEADER..STUB_CHECKSUM],
            key.as_slice(),
            &bytes[32..44],
            &bytes[..32],
        )?)
    } else {
        if bytes[32..44] != [0; 12] {
            return Err(Error::CorruptRecord);
        }
        crate::page_buffer::ZeroizingBytes::new(bytes[HEADER..STUB_CHECKSUM].to_vec())
    };
    let (sequence, previous, base, contents) = match body[0] {
        0 => {
            let record = decode_body(&body[1..])?;
            (
                record.sequence,
                record.previous,
                record.base,
                Contents::Inline(record),
            )
        }
        1 => {
            let mut cursor = Cursor(&body[1..]);
            let sequence = cursor.u64()?;
            let previous = cursor.take(32)?.try_into().unwrap();
            let base = cursor.take(32)?.try_into().unwrap();
            let reference = RootRef {
                primary: cursor.u64()?,
                mirror: cursor.u64()?,
                len: cursor.u64()?,
                digest: cursor.take(32)?.try_into().unwrap(),
            };
            validate_reference(reference)?;
            if sequence == 0
                || ((sequence == 1) != (previous == [0; 32]))
                || cursor.0.iter().any(|b| *b != 0)
            {
                return Err(Error::CorruptRecord);
            }
            (sequence, previous, base, Contents::Overflow(reference))
        }
        _ => return Err(Error::CorruptRecord),
    };
    Ok(Stub {
        sequence,
        previous,
        base,
        contents,
        digest: strong_checksum(bytes),
    })
}
fn materialize(context: &Context, storage: &impl Storage, stub: Stub) -> Result<Record> {
    let record = match stub.contents {
        Contents::Inline(record) => record,
        Contents::Overflow(reference) => {
            let bytes = Zeroizing::new(reference.read_verified(storage)?);
            context.decode(&bytes)?
        }
    };
    if record.sequence != stub.sequence
        || record.previous != stub.previous
        || record.base != stub.base
    {
        return Err(Error::CorruptRecord);
    }
    Ok(record)
}
fn select(context: &Context, storage: &impl Storage) -> Result<Record> {
    let mut stubs = Vec::with_capacity(2);
    for bank in [0, FAILURE_REGION] {
        // I/O failure is not evidence permitting fallback to an older record.
        let bytes = storage.read_at(bank + STUB_OFFSET, STUB_BYTES)?;
        if let Ok(stub) = decode(context, &bytes) {
            stubs.push(stub);
        }
    }
    stubs.sort_by_key(|stub| stub.sequence);
    let newest = stubs.pop().ok_or(Error::CorruptRecord)?;
    if let Some(old) = stubs.pop() {
        if (old.sequence == newest.sequence && old.digest != newest.digest)
            || (old.sequence != newest.sequence
                && (old.sequence.checked_add(1) != Some(newest.sequence)
                    || newest.previous != old.digest))
        {
            return Err(Error::CorruptRecord);
        }
    }
    // Only the selected stub authorizes locating its record. Losing both of its
    // overflow copies is an error, never permission to use an older reservation set.
    materialize(context, storage, newest)
}
#[cfg(test)]
mod tests;

/// Fresh-image idle marker only; not an active overflow transition or cleanup API.
pub(crate) fn initial_stub(
    archive: LockboxId,
    mode: FormatMode,
    key: Option<&[u8]>,
    base: [u8; 32],
) -> Result<Vec<u8>> {
    let context = Context::new(archive, mode, key)?;
    let record = Record {
        sequence: 1,
        previous: [0; 32],
        base,
        active: false,
        cleanup_commit: [0; 32],
        cleanup_bytes: 0,
        reservations: Vec::new(),
    };
    Ok(encode(&context, &record, None)?.stub)
}
pub(crate) fn validate_initial_idle(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    key: Option<&[u8]>,
    base: [u8; 32],
) -> Result<()> {
    let context = Context::new(archive, mode, key)?;
    let record = select(&context, storage)?;
    if record.sequence != 1
        || record.previous != [0; 32]
        || record.base != base
        || record.active
        || !record.reservations.is_empty()
        || record.cleanup_bytes != 0
        || record.cleanup_commit != [0; 32]
    {
        return Err(Error::CorruptRecord);
    }
    Ok(())
}

pub(crate) mod session;

pub(crate) fn requires_overflow(
    archive: LockboxId,
    mode: FormatMode,
    key: Option<&[u8]>,
    count: usize,
) -> Result<bool> {
    let context = Context::new(archive, mode, key)?;
    if count > MAX_RESERVATIONS {
        return Err(Error::CorruptRecord);
    }
    Ok(117 + count * 25 >= STUB_CHECKSUM - HEADER - if context.key.is_some() { 16 } else { 0 })
}
