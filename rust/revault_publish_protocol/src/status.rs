use crate::client::ClientError;
use crate::protocol::{self, ProtocolError, Reader};

const STATUS_MAGIC: &[u8; 4] = b"LBSS";
const STATUS_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents key server status.
pub struct KeyServerStatus {
    /// Represents the created carried by this record case.
    pub created: u64,
    /// Represents the received carried by this record case.
    pub received: u64,
    /// Represents the deleted carried by this record case.
    pub deleted: u64,
    /// Represents the expired carried by this record case.
    pub expired: u64,
    /// Represents the misses carried by this record case.
    pub misses: u64,
    /// Represents the live carried by this record case.
    pub live: u64,
    /// Represents the segment bytes carried by this record case.
    pub segment_bytes: u64,
    /// Represents the replication pending carried by this record case.
    pub replication_pending: u64,
    /// Represents the replication last sequence carried by this record case.
    pub replication_last_sequence: u64,
}

/// Encodes status.
pub fn encode_status(status: &KeyServerStatus) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 2 + 9 * 8);
    out.extend_from_slice(STATUS_MAGIC);
    protocol::put_u16(&mut out, STATUS_VERSION);
    protocol::put_u64(&mut out, status.created);
    protocol::put_u64(&mut out, status.received);
    protocol::put_u64(&mut out, status.deleted);
    protocol::put_u64(&mut out, status.expired);
    protocol::put_u64(&mut out, status.misses);
    protocol::put_u64(&mut out, status.live);
    protocol::put_u64(&mut out, status.segment_bytes);
    protocol::put_u64(&mut out, status.replication_pending);
    protocol::put_u64(&mut out, status.replication_last_sequence);
    out
}

/// Decodes status.
pub fn decode_status(bytes: &[u8]) -> Result<KeyServerStatus, ClientError> {
    let mut reader = Reader::new(bytes);
    let magic = reader
        .fixed_bytes(STATUS_MAGIC.len())
        .map_err(status_protocol_error)?;
    if magic != STATUS_MAGIC {
        return Err(ClientError::Protocol(ProtocolError::BadMagic));
    }
    let version = reader.u16().map_err(status_protocol_error)?;
    if version != STATUS_VERSION {
        return Err(ClientError::Protocol(ProtocolError::UnsupportedVersion));
    }
    Ok(KeyServerStatus {
        created: reader.u64().map_err(status_protocol_error)?,
        received: reader.u64().map_err(status_protocol_error)?,
        deleted: reader.u64().map_err(status_protocol_error)?,
        expired: reader.u64().map_err(status_protocol_error)?,
        misses: reader.u64().map_err(status_protocol_error)?,
        live: reader.u64().map_err(status_protocol_error)?,
        segment_bytes: reader.u64().map_err(status_protocol_error)?,
        replication_pending: reader.u64().map_err(status_protocol_error)?,
        replication_last_sequence: reader.u64().map_err(status_protocol_error)?,
    })
}

fn status_protocol_error(err: ProtocolError) -> ClientError {
    ClientError::Protocol(err)
}
