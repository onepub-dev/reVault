use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::client::{ClientError, DEFAULT_INITIAL_BACKOFF};
use crate::protocol::{ProtocolError, Status};
use crate::topology::{self, ClusterTopology, TopologyRoute};

pub(crate) trait TopologyVersionExt {
    fn if_version_for_request(&self) -> Option<u64>;
}

impl TopologyVersionExt for u64 {
    fn if_version_for_request(&self) -> Option<u64> {
        if *self == 0 {
            None
        } else {
            Some(*self)
        }
    }
}

pub(crate) fn retry_publish_error(err: &ClientError) -> bool {
    matches!(
        err,
        ClientError::Io(_)
            | ClientError::Http(_)
            | ClientError::Server {
                status: Status::StoreUnavailable | Status::InternalError,
                ..
            }
    )
}

pub(crate) fn retry_receive_or_delete_error(err: &ClientError) -> bool {
    matches!(
        err,
        ClientError::Io(_)
            | ClientError::Http(_)
            | ClientError::Server {
                status: Status::PublishNotFound | Status::StoreUnavailable | Status::InternalError,
                ..
            }
    )
}

pub(crate) fn retry_single_client_error(err: &ClientError) -> bool {
    matches!(
        err,
        ClientError::Http(_)
            | ClientError::Server {
                status: Status::StoreUnavailable | Status::RateLimited | Status::InternalError,
                ..
            }
    ) || matches!(err, ClientError::Io(io) if retry_same_endpoint_io_error(io))
}

fn retry_same_endpoint_io_error(err: &std::io::Error) -> bool {
    !matches!(
        err.kind(),
        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
    )
}

pub(crate) fn next_backoff(current: Duration, max: Duration) -> Duration {
    if current.is_zero() {
        return max.min(DEFAULT_INITIAL_BACKOFF);
    }
    current.saturating_mul(2).min(max)
}

pub(crate) fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn topology_from_tail(tail: &[u8]) -> Option<ClusterTopology> {
    if tail.is_empty() {
        return None;
    }
    topology::decode_topology(tail).ok()
}

pub(crate) fn dedupe_topology(mut topology: ClusterTopology) -> ClusterTopology {
    let mut servers = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for server in topology.servers.into_iter() {
        if seen.insert(server.id) {
            servers.push(server);
        }
    }
    topology.servers = servers;
    let mut routes = Vec::new();
    let mut route_seen = std::collections::HashSet::new();
    for route in topology.routes.into_iter() {
        let key = (route.owner_id, route.primary_id);
        if route_seen.insert(key) {
            routes.push(route);
        }
    }
    topology.routes = routes;
    topology
}

pub(crate) fn fallback_routes(server_ids: &[u8]) -> Vec<TopologyRoute> {
    if server_ids.is_empty() {
        return Vec::new();
    }
    let mut ids = server_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut routes = Vec::with_capacity(ids.len());
    for (index, owner_id) in ids.iter().copied().enumerate() {
        let failover_id = if ids.len() > 1 {
            ids[(index + 1) % ids.len()]
        } else {
            owner_id
        };
        routes.push(TopologyRoute {
            owner_id,
            primary_id: owner_id,
            failover_ids: vec![failover_id],
        });
    }
    routes
}

pub(crate) fn parse_http_response(bytes: &[u8], max_body: usize) -> Result<Vec<u8>, ClientError> {
    let header_end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| ClientError::Http("missing response headers".to_string()))?;
    let headers = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| ClientError::Http("response headers are not utf-8".to_string()))?;
    let mut lines = headers.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| ClientError::Http("missing status line".to_string()))?;
    if !status_line.starts_with("HTTP/1.1 200 ") && !status_line.starts_with("HTTP/1.0 200 ") {
        return Err(ClientError::Http(status_line.to_string()));
    }
    let body = &bytes[header_end + 4..];
    if body.len() > max_body {
        return Err(ClientError::Protocol(ProtocolError::PayloadTooLarge));
    }
    Ok(body.to_vec())
}
