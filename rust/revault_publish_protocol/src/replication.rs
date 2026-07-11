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
pub struct ReplicationRequest {
    pub authentication: Vec<u8>,
    pub event: ReplicationEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplicationEvent {
    pub origin_server_id: u8,
    pub origin_epoch: u64,
    pub origin_sequence: u64,
    pub kind: ReplicationEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplicationEventKind {
    PutPublish {
        publish_code: String,
        delete_token_hash: Vec<u8>,
        payload: Vec<u8>,
        contact_email: Option<String>,
        expires_at_unix_ms: u64,
        receive_ttl_ms: u64,
        email_verified_at_unix_ms: u64,
        max_receives: u16,
        receives: u16,
    },
    ReceiveCount {
        publish_code: String,
        receives: u16,
    },
    Tombstone {
        publish_code: String,
    },
    RateLimitBlock {
        client_ip: String,
        expires_at_unix_ms: u64,
    },
}

pub fn encode_replication_request(request: &ReplicationRequest) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(REPLICATION_MAGIC);
    protocol::put_u16(&mut payload, REPLICATION_VERSION);
    protocol::put_bytes(&mut payload, &request.authentication);
    encode_event_body(&mut payload, &request.event);
    protocol::encode_request(protocol::Operation::Replicate, &payload)
}

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
