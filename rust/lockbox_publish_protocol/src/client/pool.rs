use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::client::{
    ClientError, ContactPublish, HttpTransport, PublishClient, PublishClientPool, PublishResult,
    PublishTopologyState, ReceivedPublish, RetryPolicy, StickyPublishServer, TopologyAwareResponse,
    Transport, DEFAULT_STICKY_SERVER_TTL_MS, DEFAULT_TOPOLOGY_TTL_MS,
};
use crate::payload;
use crate::protocol::Status;
use crate::topology::{self, ClusterTopology, TopologyRoute};

use super::error::publish_state_poisoned;
use super::helpers::{
    dedupe_topology, fallback_routes, retry_publish_error, retry_receive_or_delete_error,
    unix_ms_now, TopologyVersionExt,
};
use super::http::{topology_url_from_publish_url, topology_urls_from_servers};

impl PublishClientPool<HttpTransport> {
    pub fn new<I, S>(server_urls: I) -> Result<Self, ClientError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut clients = Vec::new();
        for url in server_urls {
            clients.push(PublishClient::new(url.as_ref())?);
        }
        Self::from_clients(clients)
    }

    pub fn with_timeout(self, timeout: Duration) -> Self {
        if let Ok(mut state) = self.state.lock() {
            for client in &mut state.clients {
                client.transport.timeout = timeout;
            }
        }
        self
    }

    pub fn with_retry_policy(
        self,
        attempts: usize,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Self {
        if let Ok(mut state) = self.state.lock() {
            for client in &mut state.clients {
                client.retry_policy = RetryPolicy {
                    attempts: attempts.max(1),
                    initial_backoff,
                    max_backoff,
                };
            }
        }
        self
    }

    pub fn from_topology(topology: &ClusterTopology) -> Result<Self, ClientError> {
        topology.validate()?;
        let topology = dedupe_topology(topology.clone());
        let mut clients = Vec::new();
        let mut server_ids = Vec::new();
        for server in &topology.servers {
            clients.push(PublishClient::new(&server.url)?);
            server_ids.push(server.id);
        }
        Ok(Self {
            state: Arc::new(Mutex::new(PublishTopologyState {
                clients,
                server_ids,
                topology: Some(topology.clone()),
                routes: topology.routes.clone(),
                topology_version: topology.version,
                topology_server_urls: topology_urls_from_servers(&topology.servers),
                topology_ttl_ms: DEFAULT_TOPOLOGY_TTL_MS,
                topology_refreshed_ms: unix_ms_now(),
                sticky_server_id: None,
                sticky_until_ms: 0,
            })),
        })
    }

    pub fn discover(topology_url: &str) -> Result<Self, ClientError> {
        let bytes = HttpTransport::get_topology(topology_url).ok_or_else(|| {
            ClientError::Topology(format!("topology discovery failed for {topology_url}"))
        })?;
        let topology = topology::decode_topology(&bytes)?;
        let pool = Self::from_topology(&topology)?;
        {
            let mut state = pool.state.lock().map_err(publish_state_poisoned)?;
            let mut topology_server_urls = topology_urls_from_servers(&topology.servers);
            if let Some(topology_url) = topology_url_from_publish_url(topology_url) {
                topology_server_urls.push(topology_url);
            }
            state.topology_server_urls = dedupe_urls(topology_server_urls);
            state.topology_refreshed_ms = unix_ms_now();
        }
        Ok(pool)
    }
}

impl<T: Transport> PublishClientPool<T> {
    pub fn from_clients(clients: Vec<PublishClient<T>>) -> Result<Self, ClientError> {
        let server_ids = (0..clients.len())
            .map(|index| index as u8)
            .collect::<Vec<_>>();
        Self::from_clients_with_ids(clients, server_ids, Vec::new())
    }

    pub fn from_clients_with_ids(
        clients: Vec<PublishClient<T>>,
        server_ids: Vec<u8>,
        routes: Vec<TopologyRoute>,
    ) -> Result<Self, ClientError> {
        if clients.is_empty() {
            return Err(ClientError::Url(
                "at least one key server url is required".to_string(),
            ));
        }
        if clients.len() != server_ids.len() {
            return Err(ClientError::Topology(
                "client and server id counts differ".to_string(),
            ));
        }
        for server_id in &server_ids {
            if *server_id >= 36 {
                return Err(ClientError::Topology(format!(
                    "server id must be an index 0..35 (0..9, a..z): {server_id}"
                )));
            }
        }
        let routes = if routes.is_empty() {
            fallback_routes(&server_ids)
        } else {
            routes
        };
        Ok(Self {
            state: Arc::new(Mutex::new(PublishTopologyState {
                clients,
                server_ids,
                routes,
                topology: None,
                topology_version: 0,
                topology_server_urls: Vec::new(),
                topology_ttl_ms: DEFAULT_TOPOLOGY_TTL_MS,
                topology_refreshed_ms: 0,
                sticky_server_id: None,
                sticky_until_ms: 0,
            })),
        })
    }

    pub fn from_transports(transports: Vec<T>) -> Result<Self, ClientError> {
        let clients = transports
            .into_iter()
            .map(PublishClient::from_transport)
            .collect::<Vec<_>>();
        Self::from_clients(clients)
    }

    pub fn with_max_response_bytes(self, max_response_bytes: usize) -> Self {
        if let Ok(mut state) = self.state.lock() {
            for client in &mut state.clients {
                client.max_response_bytes = max_response_bytes;
            }
        }
        self
    }

    pub fn with_sticky_server(
        self,
        server_id: u8,
        expires_at_unix_ms: u64,
    ) -> Result<Self, ClientError> {
        self.set_sticky_server(server_id, expires_at_unix_ms)?;
        Ok(self)
    }

    pub fn set_sticky_server(
        &self,
        server_id: u8,
        expires_at_unix_ms: u64,
    ) -> Result<(), ClientError> {
        let mut state = self.state.lock().map_err(publish_state_poisoned)?;
        if !state.server_ids.contains(&server_id) {
            return Err(ClientError::Topology(format!(
                "sticky server id {server_id} is not in the current publish topology"
            )));
        }
        state.sticky_server_id = Some(server_id);
        state.sticky_until_ms = expires_at_unix_ms;
        Ok(())
    }

    pub fn sticky_server(&self) -> Result<Option<StickyPublishServer>, ClientError> {
        let snapshot = self.snapshot()?;
        let now = unix_ms_now();
        match snapshot.sticky_server_id {
            Some(server_id)
                if snapshot.sticky_until_ms > now && snapshot.server_ids.contains(&server_id) =>
            {
                Ok(Some(StickyPublishServer {
                    server_id,
                    expires_at_unix_ms: snapshot.sticky_until_ms,
                }))
            }
            _ => Ok(None),
        }
    }

    pub fn ensure_sticky_server(&self) -> Result<Option<StickyPublishServer>, ClientError> {
        if self.sticky_server()?.is_none() {
            let _ = self.selection_offset();
        }
        self.sticky_server()
    }

    pub fn publish_payload(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        payload: &[u8],
    ) -> Result<PublishResult, ClientError> {
        self.publish_payload_with_email(ttl_seconds, max_receives, payload, None)
    }

    pub fn publish_payload_with_email(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        payload: &[u8],
        verification_email: Option<&str>,
    ) -> Result<PublishResult, ClientError> {
        self.try_clients_from(
            self.selection_offset(),
            |client, topology_version| {
                client.publish_payload_with_email_with_version(
                    ttl_seconds,
                    max_receives,
                    payload,
                    verification_email,
                    topology_version,
                )
            },
            retry_publish_error,
        )
    }

    pub fn publish_contact(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        contact: ContactPublish<'_>,
    ) -> Result<PublishResult, ClientError> {
        let payload = payload::encode_contact_publish(
            contact.identity,
            contact.public_key,
            contact.signing_public_key,
            contact.fingerprint,
            contact.publish_nonce,
            contact.created_at_unix_ms,
            contact.expires_at_unix_ms,
        );
        self.publish_payload_with_email(
            ttl_seconds,
            max_receives,
            &payload,
            contact.verification_email,
        )
    }

    pub fn receive(&self, publish_code: &str) -> Result<ReceivedPublish, ClientError> {
        self.try_clients_for_code(
            publish_code,
            |client, topology_version| client.receive_with_version(publish_code, topology_version),
            retry_receive_or_delete_error,
        )
    }

    pub fn delete(&self, publish_code: &str, delete_token: &[u8]) -> Result<bool, ClientError> {
        let snapshot = self.snapshot()?;
        if snapshot.clients.is_empty() {
            return Err(ClientError::Url(
                "at least one key server url is required".to_string(),
            ));
        }
        let mut snapshot = self.discover_topology_if_stale(&snapshot)?;
        let mut last_error = None;
        for _ in 0..2 {
            let topology_version = snapshot.topology_version.if_version_for_request();
            let clients = self.clients_for_code(publish_code, &snapshot);
            for client in clients {
                match client.delete_with_version(publish_code, delete_token, topology_version) {
                    Ok(response) => {
                        if let Some(topology) = response.topology {
                            let _ = self.apply_topology_update(topology);
                        }
                        if response.value {
                            return Ok(true);
                        }
                        last_error = Some(ClientError::Server {
                            status: Status::PublishNotFound,
                            message: "delete not performed on this server".to_string(),
                        });
                    }
                    Err(err) if retry_receive_or_delete_error(&err) => {
                        last_error = Some(err);
                    }
                    Err(err) => return Err(err),
                }
            }
            let current = self.snapshot()?;
            if !self.refresh_topology_from_peers(&current) {
                break;
            }
            snapshot = self.snapshot()?;
        }
        match last_error {
            Some(ClientError::Server {
                status: Status::PublishNotFound,
                ..
            }) => Ok(false),
            Some(err) => Err(err),
            None => Ok(false),
        }
    }

    pub(crate) fn try_clients_from<R>(
        &self,
        start: usize,
        mut call: impl FnMut(
            &PublishClient<T>,
            Option<u64>,
        ) -> Result<TopologyAwareResponse<R>, ClientError>,
        retry: impl Fn(&ClientError) -> bool,
    ) -> Result<R, ClientError> {
        let snapshot = self.snapshot()?;
        if snapshot.clients.is_empty() {
            return Err(ClientError::Url(
                "at least one key server url is required".to_string(),
            ));
        }
        let mut snapshot = self.discover_topology_if_stale(&snapshot)?;
        let mut last_error = None;
        for _ in 0..2 {
            let topology_version = snapshot.topology_version.if_version_for_request();
            let clients = snapshot.clients.clone();
            for offset in 0..clients.len() {
                let index = (start + offset) % clients.len();
                match call(&clients[index], topology_version) {
                    Ok(response) => {
                        if let Some(topology) = response.topology {
                            let _ = self.apply_topology_update(topology);
                        }
                        return Ok(response.value);
                    }
                    Err(err) if retry(&err) => last_error = Some(err),
                    Err(err) => return Err(err),
                }
            }
            let current = self.snapshot()?;
            if !self.refresh_topology_from_peers(&current) {
                break;
            }
            snapshot = self.snapshot()?;
        }
        Err(last_error.unwrap_or_else(|| {
            ClientError::Url("at least one key server url is required".to_string())
        }))
    }

    pub(crate) fn selection_offset(&self) -> usize {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        let now = unix_ms_now();
        if state.clients.len() <= 1 {
            state.sticky_server_id = state.server_ids.first().copied();
            state.sticky_until_ms = now.saturating_add(DEFAULT_STICKY_SERVER_TTL_MS);
            return 0;
        }
        if let Some(sticky_server_id) = state.sticky_server_id {
            if state.sticky_until_ms > now {
                if let Some(index) = state
                    .server_ids
                    .iter()
                    .position(|server_id| *server_id == sticky_server_id)
                {
                    return index;
                }
            }
        }
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.subsec_nanos() as usize % state.clients.len())
            .unwrap_or(0);
        state.sticky_server_id = state.server_ids.get(index).copied();
        state.sticky_until_ms = now.saturating_add(DEFAULT_STICKY_SERVER_TTL_MS);
        index
    }
}

fn dedupe_urls<T: AsRef<str>>(values: impl IntoIterator<Item = T>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for value in values {
        let value = value.as_ref().to_string();
        if seen.insert(value.clone()) {
            out.push(value);
        }
    }
    out
}
