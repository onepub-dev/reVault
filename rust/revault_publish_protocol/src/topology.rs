use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::client::ClientError;
use crate::protocol::{self, ProtocolError, Reader};
use sha2::{Digest, Sha256};

mod cache;
mod registration;

#[cfg(test)]
mod tests;

pub use cache::{read_topology_cache, write_topology_cache};
pub use registration::{decode_topology_registration, encode_topology_registration};

pub(crate) const TOPOLOGY_MAGIC: &[u8; 4] = b"LBST";
pub(crate) const TOPOLOGY_VERSION: u16 = 2;
pub(crate) const TOPOLOGY_CACHE_MAGIC: &[u8; 4] = b"LBTC";
pub(crate) const TOPOLOGY_CACHE_VERSION: u16 = 1;
pub(crate) const TOPOLOGY_REGISTRATION_MAGIC: &[u8; 4] = b"LBTR";
pub(crate) const TOPOLOGY_REGISTRATION_VERSION: u16 = 1;
pub(crate) const STATUS_ACTIVE: u8 = 1;
pub(crate) const STATUS_STANDBY: u8 = 2;
pub(crate) const STATUS_PROMOTED: u8 = 3;
pub(crate) const STATUS_DISABLED: u8 = 4;
const SHARE_CODE_SERVER_ID_ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const SHARE_CODE_LEN: usize = 14;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents cluster topology.
pub struct ClusterTopology {
    /// Represents the cluster id carried by this record case.
    pub cluster_id: String,
    /// Represents the version carried by this record case.
    pub version: u64,
    /// Represents the servers carried by this record case.
    pub servers: Vec<TopologyServer>,
    /// Represents the routes carried by this record case.
    pub routes: Vec<TopologyRoute>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents topology server.
pub struct TopologyServer {
    /// Represents the id carried by this record case.
    pub id: u8,
    /// Represents the url carried by this record case.
    pub url: String,
    /// Represents the status carried by this record case.
    pub status: ServerStatus,
    /// Represents the last seen ms carried by this record case.
    pub last_seen_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents topology registration.
pub struct TopologyRegistration {
    /// Represents the cluster id carried by this record case.
    pub cluster_id: String,
    /// Represents the server id carried by this record case.
    pub server_id: u8,
    /// Represents the server url carried by this record case.
    pub server_url: String,
    /// Represents the status carried by this record case.
    pub status: ServerStatus,
    /// Represents the security token carried by this record case.
    pub security_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents topology route.
pub struct TopologyRoute {
    /// Represents the owner id carried by this record case.
    pub owner_id: u8,
    /// Represents the primary id carried by this record case.
    pub primary_id: u8,
    /// Represents the failover ids carried by this record case.
    pub failover_ids: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents server status.
pub enum ServerStatus {
    /// Represents the active case.
    Active,
    /// Represents the standby case.
    Standby,
    /// Represents the promoted case.
    Promoted,
    /// Represents the disabled case.
    Disabled,
}

impl ClusterTopology {
    /// Returns the single server.
    pub fn single_server(server_id: u8, url: impl Into<String>) -> Self {
        Self {
            cluster_id: "default".to_string(),
            version: 1,
            servers: vec![TopologyServer {
                id: server_id,
                url: url.into(),
                status: ServerStatus::Active,
                last_seen_ms: None,
            }],
            routes: vec![TopologyRoute {
                owner_id: server_id,
                primary_id: server_id,
                failover_ids: Vec::new(),
            }],
        }
    }

    /// Returns the with filtered stale servers.
    pub fn with_filtered_stale_servers(&self, stale_after_ms: u64) -> ClusterTopology {
        if stale_after_ms == 0 {
            return self.clone();
        }
        let now_ms = unix_ms(SystemTime::now());
        let active_server_ids = self
            .servers
            .iter()
            .filter(|server| !is_topology_server_stale(server, now_ms, stale_after_ms))
            .filter(|server| {
                matches!(
                    server.status,
                    ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
                )
            })
            .map(|server| server.id)
            .collect::<HashSet<_>>();
        let servers = self
            .servers
            .iter()
            .filter(|server| active_server_ids.contains(&server.id))
            .cloned()
            .collect::<Vec<_>>();
        let mut routes = self
            .routes
            .iter()
            .filter(|route| active_server_ids.contains(&route.owner_id))
            .filter(|route| active_server_ids.contains(&route.primary_id))
            .filter(|route| {
                route
                    .failover_ids
                    .iter()
                    .all(|id| active_server_ids.contains(id))
            })
            .cloned()
            .collect::<Vec<_>>();
        if routes.is_empty() {
            routes = build_ring_routes(&servers);
        }
        ClusterTopology {
            cluster_id: self.cluster_id.clone(),
            version: self.version,
            servers,
            routes,
        }
    }

    /// Returns the validate.
    pub fn validate(&self) -> Result<(), ClientError> {
        if self.cluster_id.is_empty() {
            return Err(ClientError::Topology(
                "topology cluster_id must not be empty".to_string(),
            ));
        }
        if self.servers.is_empty() {
            return Err(ClientError::Topology(
                "topology must include at least one server".to_string(),
            ));
        }
        for server in &self.servers {
            validate_server_id(server.id)?;
            if server.url.is_empty() {
                return Err(ClientError::Topology(format!(
                    "server {} url must not be empty",
                    server.id
                )));
            }
        }
        for route in &self.routes {
            validate_server_id(route.owner_id)?;
            validate_server_id(route.primary_id)?;
            if self.server(route.primary_id).is_none() {
                return Err(ClientError::Topology(format!(
                    "route owner {} references unknown primary {}",
                    route.owner_id, route.primary_id
                )));
            }
            for failover_id in &route.failover_ids {
                validate_server_id(*failover_id)?;
                if self.server(*failover_id).is_none() {
                    return Err(ClientError::Topology(format!(
                        "route owner {} references unknown failover {}",
                        route.owner_id, failover_id
                    )));
                }
            }
        }
        Ok(())
    }

    /// Returns the server.
    pub fn server(&self, id: u8) -> Option<&TopologyServer> {
        self.servers.iter().find(|server| server.id == id)
    }

    /// Returns the route.
    pub fn route(&self, owner_id: u8) -> Option<&TopologyRoute> {
        self.routes.iter().find(|route| route.owner_id == owner_id)
    }

    /// Returns the urls for publish code.
    pub fn urls_for_publish_code(&self, publish_code: &str) -> Vec<String> {
        let Some(owner_id) = publish_code_owner_id(publish_code) else {
            return self.active_urls();
        };
        let Some(route) = self.route(owner_id) else {
            return self.active_urls();
        };
        let mut out = Vec::new();
        if let Some(server) = self.server(route.primary_id) {
            out.push(server.url.clone());
        }
        for failover_id in &route.failover_ids {
            if let Some(server) = self.server(*failover_id) {
                if !out.iter().any(|url| url == &server.url) {
                    out.push(server.url.clone());
                }
            }
        }
        for url in self.active_urls() {
            if !out.iter().any(|existing| existing == &url) {
                out.push(url);
            }
        }
        out
    }

    /// Returns the verification email owner id.
    pub fn verification_email_owner_id(&self, normalized_email: &str) -> Option<u8> {
        self.verification_email_server_ids(normalized_email)
            .into_iter()
            .next()
    }

    /// Returns the verification email server ids.
    pub fn verification_email_server_ids(&self, normalized_email: &str) -> Vec<u8> {
        let mut active_ids = self
            .servers
            .iter()
            .filter(|server| {
                matches!(
                    server.status,
                    ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
                )
            })
            .map(|server| server.id)
            .collect::<Vec<_>>();
        active_ids.sort_unstable();
        active_ids.dedup();
        if active_ids.is_empty() {
            return Vec::new();
        }

        let mut hasher = Sha256::new();
        hasher.update(b"revault-verification-email-owner-v1");
        hasher.update((self.cluster_id.len() as u64).to_be_bytes());
        hasher.update(self.cluster_id.as_bytes());
        hasher.update((normalized_email.len() as u64).to_be_bytes());
        hasher.update(normalized_email.as_bytes());
        let digest = hasher.finalize();
        let mut value_bytes = [0_u8; 8];
        value_bytes.copy_from_slice(&digest[..8]);
        let owner_index = u64::from_be_bytes(value_bytes) as usize % active_ids.len();
        let owner_id = active_ids[owner_index];
        let route = self.route(owner_id);
        let primary_id = route
            .map(|route| route.primary_id)
            .filter(|primary_id| active_ids.contains(primary_id))
            .unwrap_or(owner_id);
        let mut out = vec![primary_id];
        if let Some(failover_id) = route
            .into_iter()
            .flat_map(|route| route.failover_ids.iter().copied())
            .find(|failover_id| *failover_id != primary_id && active_ids.contains(failover_id))
        {
            out.push(failover_id);
            return out;
        }
        if active_ids.len() > 1 {
            let primary_index = active_ids
                .iter()
                .position(|server_id| *server_id == primary_id)
                .unwrap_or(owner_index);
            out.push(active_ids[(primary_index + 1) % active_ids.len()]);
        }
        out
    }

    /// Returns the urls for verification email.
    pub fn urls_for_verification_email(&self, normalized_email: &str) -> Vec<String> {
        let owner_ids = self.verification_email_server_ids(normalized_email);
        if owner_ids.is_empty() {
            return self.active_urls();
        }
        owner_ids
            .into_iter()
            .filter_map(|server_id| self.server(server_id))
            .map(|server| server.url.clone())
            .collect()
    }

    /// Returns the active urls.
    pub fn active_urls(&self) -> Vec<String> {
        self.servers
            .iter()
            .filter(|server| {
                matches!(
                    server.status,
                    ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
                )
            })
            .map(|server| server.url.clone())
            .collect()
    }

    /// Returns the active urls with ttl.
    pub fn active_urls_with_ttl(&self, stale_after_ms: u64) -> Vec<String> {
        self.with_filtered_stale_servers(stale_after_ms)
            .servers
            .into_iter()
            .filter(|server| {
                matches!(
                    server.status,
                    ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
                )
            })
            .map(|server| server.url)
            .collect()
    }

    /// Returns the routes for owner.
    pub fn routes_for_owner(&self, owner_id: u8) -> Option<&TopologyRoute> {
        self.route(owner_id)
    }
}

/// Returns the build ring routes.
pub fn build_ring_routes(servers: &[TopologyServer]) -> Vec<TopologyRoute> {
    let mut active_ids = servers
        .iter()
        .filter(|server| {
            matches!(
                server.status,
                ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
            )
        })
        .map(|server| server.id)
        .collect::<Vec<_>>();
    active_ids.sort_unstable();
    active_ids.dedup();
    if active_ids.is_empty() {
        return Vec::new();
    }
    let mut routes = Vec::with_capacity(active_ids.len());
    for (index, owner_id) in active_ids.iter().enumerate() {
        let failover_ids = if active_ids.len() > 1 {
            vec![active_ids[(index + 1) % active_ids.len()]]
        } else {
            vec![*owner_id]
        };
        routes.push(TopologyRoute {
            owner_id: *owner_id,
            primary_id: *owner_id,
            failover_ids,
        });
    }
    routes
}

/// Reports whether topology server stale.
pub fn is_topology_server_stale(server: &TopologyServer, now_ms: u64, stale_after_ms: u64) -> bool {
    if stale_after_ms == 0 {
        return false;
    }
    match server.last_seen_ms {
        Some(last_seen_ms) => now_ms.saturating_sub(last_seen_ms) > stale_after_ms,
        None => false,
    }
}

/// Returns the parse publish locator.
pub fn parse_publish_locator(value: &str) -> Option<(u8, u8)> {
    publish_code_locator(value)
}

/// Encodes topology.
pub fn encode_topology(topology: &ClusterTopology) -> Result<Vec<u8>, ClientError> {
    topology.validate()?;
    if topology.servers.len() > u16::MAX as usize || topology.routes.len() > u16::MAX as usize {
        return Err(ClientError::Topology(
            "topology has too many servers or routes".to_string(),
        ));
    }
    let mut out = Vec::new();
    out.extend_from_slice(TOPOLOGY_MAGIC);
    protocol::put_u16(&mut out, TOPOLOGY_VERSION);
    protocol::put_string(&mut out, &topology.cluster_id);
    protocol::put_u64(&mut out, topology.version);
    protocol::put_u16(&mut out, topology.servers.len() as u16);
    for server in &topology.servers {
        out.push(server.id);
        out.push(server_status_to_u8(&server.status));
        out.push(if server.last_seen_ms.is_some() { 1 } else { 0 });
        if let Some(last_seen_ms) = server.last_seen_ms {
            protocol::put_u64(&mut out, last_seen_ms);
        }
        protocol::put_string(&mut out, &server.url);
    }
    protocol::put_u16(&mut out, topology.routes.len() as u16);
    for route in &topology.routes {
        if route.failover_ids.len() > u16::MAX as usize {
            return Err(ClientError::Topology(
                "topology route has too many failover ids".to_string(),
            ));
        }
        out.push(route.owner_id);
        out.push(route.primary_id);
        protocol::put_u16(&mut out, route.failover_ids.len() as u16);
        out.extend_from_slice(&route.failover_ids);
    }
    Ok(out)
}

/// Decodes topology.
pub fn decode_topology(bytes: &[u8]) -> Result<ClusterTopology, ClientError> {
    let mut reader = Reader::new(bytes);
    let magic = reader
        .fixed_bytes(TOPOLOGY_MAGIC.len())
        .map_err(topology_protocol_error)?;
    if magic != TOPOLOGY_MAGIC {
        return Err(ClientError::Topology(
            "topology document has invalid magic".to_string(),
        ));
    }
    let version = reader.u16().map_err(topology_protocol_error)?;
    if version != TOPOLOGY_VERSION {
        return Err(ClientError::Topology(format!(
            "topology version {version} is not supported"
        )));
    }
    let cluster_id = reader.string().map_err(topology_protocol_error)?;
    let topology_version = reader.u64().map_err(topology_protocol_error)?;
    let server_count = reader.u16().map_err(topology_protocol_error)? as usize;
    let mut servers = Vec::with_capacity(server_count);
    for _ in 0..server_count {
        let id = reader.u8().map_err(topology_protocol_error)?;
        let status = server_status_from_u8(reader.u8().map_err(topology_protocol_error)?)?;
        let has_last_seen = reader.u8().map_err(topology_protocol_error)? != 0;
        let last_seen_ms = if has_last_seen {
            Some(reader.u64().map_err(topology_protocol_error)?)
        } else {
            None
        };
        let url = reader.string().map_err(topology_protocol_error)?;
        servers.push(TopologyServer {
            id,
            url,
            status,
            last_seen_ms,
        });
    }
    let route_count = reader.u16().map_err(topology_protocol_error)? as usize;
    let mut routes = Vec::with_capacity(route_count);
    for _ in 0..route_count {
        let owner_id = reader.u8().map_err(topology_protocol_error)?;
        let primary_id = reader.u8().map_err(topology_protocol_error)?;
        let failover_count = reader.u16().map_err(topology_protocol_error)? as usize;
        let mut failover_ids = Vec::with_capacity(failover_count);
        for _ in 0..failover_count {
            failover_ids.push(reader.u8().map_err(topology_protocol_error)?);
        }
        routes.push(TopologyRoute {
            owner_id,
            primary_id,
            failover_ids,
        });
    }
    let topology = ClusterTopology {
        cluster_id,
        version: topology_version,
        servers,
        routes,
    };
    topology.validate()?;
    Ok(topology)
}

/// Returns the publish code owner id.
pub fn publish_code_owner_id(publish_code: &str) -> Option<u8> {
    if publish_code.len() != SHARE_CODE_LEN {
        return None;
    }
    parse_publish_code_server_id(*publish_code.as_bytes().first()?)
}

/// Returns the publish code locator.
pub fn publish_code_locator(publish_code: &str) -> Option<(u8, u8)> {
    let bytes = publish_code.as_bytes();
    if bytes.len() != SHARE_CODE_LEN {
        return None;
    }
    let owner_id = parse_publish_code_server_id(*bytes.first()?)?;
    let secondary_id = parse_publish_code_server_id(*bytes.get(1)?)?;
    Some((owner_id, secondary_id))
}

/// Returns the publish code server id char.
pub fn publish_code_server_id_char(id: u8) -> Option<u8> {
    SHARE_CODE_SERVER_ID_ALPHABET.get(id as usize).copied()
}

fn parse_publish_code_server_id(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'z' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn validate_server_id(id: u8) -> Result<(), ClientError> {
    if id < SHARE_CODE_SERVER_ID_ALPHABET.len() as u8 {
        Ok(())
    } else {
        Err(ClientError::Topology(format!(
            "server id must be an index 0..35 (0..9, a..z): {id}"
        )))
    }
}

pub(crate) fn server_status_to_u8(status: &ServerStatus) -> u8 {
    match status {
        ServerStatus::Active => STATUS_ACTIVE,
        ServerStatus::Standby => STATUS_STANDBY,
        ServerStatus::Promoted => STATUS_PROMOTED,
        ServerStatus::Disabled => STATUS_DISABLED,
    }
}

pub(crate) fn server_status_from_u8(value: u8) -> Result<ServerStatus, ClientError> {
    match value {
        STATUS_ACTIVE => Ok(ServerStatus::Active),
        STATUS_STANDBY => Ok(ServerStatus::Standby),
        STATUS_PROMOTED => Ok(ServerStatus::Promoted),
        STATUS_DISABLED => Ok(ServerStatus::Disabled),
        _ => Err(ClientError::Topology(format!(
            "unknown topology server status {value}"
        ))),
    }
}

pub(crate) fn topology_protocol_error(err: ProtocolError) -> ClientError {
    ClientError::Topology(err.to_string())
}

pub(crate) fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
