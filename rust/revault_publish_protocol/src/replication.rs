use crate::client::ClientError;
use crate::protocol::{self, ProtocolError, Reader};
use sha2::{Digest, Sha256};

const REPLICATION_MAGIC: &[u8; 4] = b"LBSX";
const REPLICATION_VERSION: u16 = 1;
const EVENT_PUT_PUBLISH: u16 = 1;
const EVENT_RECEIVE_COUNT: u16 = 2;
const EVENT_TOMBSTONE: u16 = 3;
const EVENT_RATE_LIMIT_BLOCK: u16 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents replication request.
pub struct ReplicationRequest {
    /// Represents the authentication carried by this record case.
    pub authentication: Vec<u8>,
    /// Represents the event carried by this record case.
    pub event: ReplicationEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents replication event.
pub struct ReplicationEvent {
    /// Represents the origin server id carried by this record case.
    pub origin_server_id: u8,
    /// Represents the origin epoch carried by this record case.
    pub origin_epoch: u64,
    /// Represents the origin sequence carried by this record case.
    pub origin_sequence: u64,
    /// Represents the kind carried by this record case.
    pub kind: ReplicationEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents replication event kind.
pub enum ReplicationEventKind {
    /// Represents the put publish case.
    PutPublish {
        /// Represents the publish code carried by this record case.
        publish_code: String,
        /// Represents the delete token hash carried by this record case.
        delete_token_hash: Vec<u8>,
        /// Represents the payload carried by this record case.
        payload: Vec<u8>,
        /// Represents the contact email carried by this record case.
        contact_email: Option<String>,
        /// Represents the expires at unix ms carried by this record case.
        expires_at_unix_ms: u64,
        /// Represents the receive ttl ms carried by this record case.
        receive_ttl_ms: u64,
        /// Represents the email verified at unix ms carried by this record case.
        email_verified_at_unix_ms: u64,
        /// Represents the max receives carried by this record case.
        max_receives: u16,
        /// Represents the receives carried by this record case.
        receives: u16,
    },
    /// Represents the receive count case.
    ReceiveCount {
        /// Represents the publish code carried by this record case.
        publish_code: String,
        /// Represents the receives carried by this record case.
        receives: u16,
    },
    /// Represents the tombstone case.
    Tombstone {
        /// Represents the publish code carried by this record case.
        publish_code: String,
    },
    /// Represents the rate limit block case.
    RateLimitBlock {
        /// Represents the client ip carried by this record case.
        client_ip: String,
        /// Represents the expires at unix ms carried by this record case.
        expires_at_unix_ms: u64,
    },
}

/// Encodes replication request.
pub fn encode_replication_request(request: &ReplicationRequest) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(REPLICATION_MAGIC);
    protocol::put_u16(&mut payload, REPLICATION_VERSION);
    protocol::put_bytes(&mut payload, &request.authentication);
    encode_event_body(&mut payload, &request.event);
    protocol::encode_request(protocol::Operation::Replicate, &payload)
}

/// Returns the sign replication event.
pub fn sign_replication_event(token: &[u8], event: &ReplicationEvent) -> Vec<u8> {
    let mut event_body = Vec::new();
    encode_event_body(&mut event_body, event);
    let mut hasher = Sha256::new();
    hasher.update(b"LBSX-auth-v1");
    hasher.update(token);
    hasher.update(&event_body);
    hasher.finalize().to_vec()
}

fn encode_event_body(out: &mut Vec<u8>, event: &ReplicationEvent) {
    out.push(event.origin_server_id);
    protocol::put_u64(out, event.origin_epoch);
    protocol::put_u64(out, event.origin_sequence);
    match &event.kind {
        ReplicationEventKind::PutPublish {
            publish_code,
            delete_token_hash,
            payload,
            contact_email,
            expires_at_unix_ms,
            receive_ttl_ms,
            email_verified_at_unix_ms,
            max_receives,
            receives,
        } => {
            protocol::put_u16(out, EVENT_PUT_PUBLISH);
            protocol::put_string(out, publish_code);
            protocol::put_bytes(out, delete_token_hash);
            protocol::put_bytes(out, payload);
            protocol::put_u64(out, *expires_at_unix_ms);
            protocol::put_u64(out, *receive_ttl_ms);
            protocol::put_u64(out, *email_verified_at_unix_ms);
            protocol::put_u16(out, *max_receives);
            protocol::put_u16(out, *receives);
            if let Some(email) = contact_email {
                protocol::put_string(out, email);
            }
        }
        ReplicationEventKind::ReceiveCount {
            publish_code,
            receives,
        } => {
            protocol::put_u16(out, EVENT_RECEIVE_COUNT);
            protocol::put_string(out, publish_code);
            protocol::put_u16(out, *receives);
        }
        ReplicationEventKind::Tombstone { publish_code } => {
            protocol::put_u16(out, EVENT_TOMBSTONE);
            protocol::put_string(out, publish_code);
        }
        ReplicationEventKind::RateLimitBlock {
            client_ip,
            expires_at_unix_ms,
        } => {
            protocol::put_u16(out, EVENT_RATE_LIMIT_BLOCK);
            protocol::put_string(out, client_ip);
            protocol::put_u64(out, *expires_at_unix_ms);
        }
    }
}

/// Decodes replication request.
pub fn decode_replication_request(bytes: &[u8]) -> Result<ReplicationRequest, ClientError> {
    let mut reader = Reader::new(bytes);
    let magic = reader
        .fixed_bytes(REPLICATION_MAGIC.len())
        .map_err(replication_protocol_error)?;
    if magic != REPLICATION_MAGIC {
        return Err(ClientError::Replication(
            "replication request has invalid magic".to_string(),
        ));
    }
    let version = reader.u16().map_err(replication_protocol_error)?;
    if version != REPLICATION_VERSION {
        return Err(ClientError::Replication(format!(
            "replication version {version} is not supported"
        )));
    }
    let authentication = reader.bytes().map_err(replication_protocol_error)?;
    let origin_server_id = reader.u8().map_err(replication_protocol_error)?;
    let origin_epoch = reader.u64().map_err(replication_protocol_error)?;
    let origin_sequence = reader.u64().map_err(replication_protocol_error)?;
    let event_type = reader.u16().map_err(replication_protocol_error)?;
    let kind = match event_type {
        EVENT_PUT_PUBLISH => ReplicationEventKind::PutPublish {
            publish_code: reader.string().map_err(replication_protocol_error)?,
            delete_token_hash: reader.bytes().map_err(replication_protocol_error)?,
            payload: reader.bytes().map_err(replication_protocol_error)?,
            expires_at_unix_ms: reader.u64().map_err(replication_protocol_error)?,
            receive_ttl_ms: reader.u64().map_err(replication_protocol_error)?,
            email_verified_at_unix_ms: reader.u64().map_err(replication_protocol_error)?,
            max_receives: reader.u16().map_err(replication_protocol_error)?,
            receives: reader.u16().map_err(replication_protocol_error)?,
            contact_email: if reader.is_done() {
                None
            } else {
                Some(reader.string().map_err(replication_protocol_error)?)
            },
        },
        EVENT_RECEIVE_COUNT => ReplicationEventKind::ReceiveCount {
            publish_code: reader.string().map_err(replication_protocol_error)?,
            receives: reader.u16().map_err(replication_protocol_error)?,
        },
        EVENT_TOMBSTONE => ReplicationEventKind::Tombstone {
            publish_code: reader.string().map_err(replication_protocol_error)?,
        },
        EVENT_RATE_LIMIT_BLOCK => ReplicationEventKind::RateLimitBlock {
            client_ip: reader.string().map_err(replication_protocol_error)?,
            expires_at_unix_ms: reader.u64().map_err(replication_protocol_error)?,
        },
        _ => {
            return Err(ClientError::Replication(format!(
                "unknown replication event type {event_type}"
            )))
        }
    };
    Ok(ReplicationRequest {
        authentication,
        event: ReplicationEvent {
            origin_server_id,
            origin_epoch,
            origin_sequence,
            kind,
        },
    })
}

fn replication_protocol_error(err: ProtocolError) -> ClientError {
    ClientError::Replication(err.to_string())
}
