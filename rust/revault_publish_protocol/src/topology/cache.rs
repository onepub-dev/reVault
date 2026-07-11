use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::client::ClientError;
use crate::protocol::{self, Reader};
use crate::topology::{
    decode_topology, topology_protocol_error, unix_ms, ClusterTopology, TOPOLOGY_CACHE_MAGIC,
    TOPOLOGY_CACHE_VERSION,
};

pub fn write_topology_cache(
    path: impl AsRef<Path>,
    topology: &ClusterTopology,
) -> Result<(), ClientError> {
    let topology = crate::topology::encode_topology(topology)?;
    let mut out = Vec::new();
    out.extend_from_slice(TOPOLOGY_CACHE_MAGIC);
    protocol::put_u16(&mut out, TOPOLOGY_CACHE_VERSION);
    protocol::put_u64(&mut out, unix_ms(SystemTime::now()));
    protocol::put_bytes(&mut out, &topology);
    fs::write(path, out).map_err(ClientError::Io)
}

pub fn read_topology_cache(
    path: impl AsRef<Path>,
    max_age: Duration,
) -> Result<Option<ClusterTopology>, ClientError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(ClientError::Io(err)),
    };
    let mut reader = Reader::new(&bytes);
    let magic = reader
        .fixed_bytes(TOPOLOGY_CACHE_MAGIC.len())
        .map_err(topology_protocol_error)?;
    if magic != TOPOLOGY_CACHE_MAGIC {
        return Err(ClientError::Topology(
            "topology cache has invalid magic".to_string(),
        ));
    }
    let version = reader.u16().map_err(topology_protocol_error)?;
    if version != TOPOLOGY_CACHE_VERSION {
        return Err(ClientError::Topology(format!(
            "topology cache version {version} is not supported"
        )));
    }
    let fetched_at_ms = reader.u64().map_err(topology_protocol_error)?;
    let now_ms = unix_ms(SystemTime::now());
    if now_ms.saturating_sub(fetched_at_ms) > max_age.as_millis() as u64 {
        return Ok(None);
    }
    let topology = reader.bytes().map_err(topology_protocol_error)?;
    decode_topology(&topology).map(Some)
}
