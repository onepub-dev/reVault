use crate::client::ClientError;
use crate::protocol::{self, Reader};
use crate::topology::{
    server_status_from_u8, server_status_to_u8, topology_protocol_error, TopologyRegistration,
    TOPOLOGY_REGISTRATION_MAGIC, TOPOLOGY_REGISTRATION_VERSION,
};

pub fn encode_topology_registration(
    registration: &TopologyRegistration,
) -> Result<Vec<u8>, ClientError> {
    let mut out = Vec::new();
    out.extend_from_slice(TOPOLOGY_REGISTRATION_MAGIC);
    protocol::put_u16(&mut out, TOPOLOGY_REGISTRATION_VERSION);
    protocol::put_string(&mut out, &registration.cluster_id);
    protocol::put_u64(&mut out, registration.server_id as u64);
    protocol::put_string(&mut out, &registration.server_url);
    out.push(server_status_to_u8(&registration.status));
    protocol::put_string(&mut out, &registration.security_token);
    Ok(out)
}

pub fn decode_topology_registration(bytes: &[u8]) -> Result<TopologyRegistration, ClientError> {
    let mut reader = Reader::new(bytes);
    let magic = reader
        .fixed_bytes(TOPOLOGY_REGISTRATION_MAGIC.len())
        .map_err(topology_protocol_error)?;
    if magic != TOPOLOGY_REGISTRATION_MAGIC {
        return Err(ClientError::Topology(
            "topology registration document has invalid magic".to_string(),
        ));
    }
    let version = reader.u16().map_err(topology_protocol_error)?;
    if version != TOPOLOGY_REGISTRATION_VERSION {
        return Err(ClientError::Topology(format!(
            "topology registration version {version} is not supported"
        )));
    }
    let cluster_id = reader.string().map_err(topology_protocol_error)?;
    let server_id = reader.u64().map_err(topology_protocol_error)? as u8;
    let server_url = reader.string().map_err(topology_protocol_error)?;
    let status = server_status_from_u8(reader.u8().map_err(topology_protocol_error)?)?;
    let security_token = reader.string().map_err(topology_protocol_error)?;
    Ok(TopologyRegistration {
        cluster_id,
        server_id,
        server_url,
        status,
        security_token,
    })
}
