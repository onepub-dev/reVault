use std::collections::HashSet;

use crate::client::{
    ClientError, PublishClient, PublishClientPool, TopologyStateSnapshot, Transport,
};
use crate::topology::{self, build_ring_routes, ClusterTopology};

use super::error::publish_state_poisoned;
use super::helpers::{dedupe_topology, TopologyVersionExt};

impl<T: Transport> PublishClientPool<T> {
    pub(crate) fn try_clients_for_code<R>(
        &self,
        publish_code: &str,
        mut call: impl FnMut(
            &PublishClient<T>,
            Option<u64>,
        ) -> Result<super::TopologyAwareResponse<R>, ClientError>,
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
            let clients = self.clients_for_code(publish_code, &snapshot);
            for client in clients {
                match call(&client, topology_version) {
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

    pub(crate) fn snapshot(&self) -> Result<TopologyStateSnapshot<T>, ClientError> {
        Ok(self
            .state
            .lock()
            .map_err(publish_state_poisoned)?
            .snapshot())
    }

    pub(crate) fn clients_for_code(
        &self,
        publish_code: &str,
        snapshot: &TopologyStateSnapshot<T>,
    ) -> Vec<PublishClient<T>> {
        let mut preferred_ids = Vec::new();
        if let Some((owner_id, secondary_id)) = topology::publish_code_locator(publish_code) {
            if let Some(route) = snapshot
                .routes
                .iter()
                .find(|route| route.owner_id == owner_id)
            {
                preferred_ids.push(route.primary_id);
                preferred_ids.extend(route.failover_ids.iter().copied());
            }
            if preferred_ids.is_empty() {
                preferred_ids.push(owner_id);
                if secondary_id != owner_id {
                    preferred_ids.push(secondary_id);
                }
            }
        }
        let mut selected = HashSet::new();
        let mut out = Vec::with_capacity(snapshot.clients.len());
        for preferred_id in preferred_ids {
            if let Some((index, _)) = snapshot
                .server_ids
                .iter()
                .enumerate()
                .find(|(_, server_id)| **server_id == preferred_id)
            {
                if selected.insert(snapshot.server_ids[index]) {
                    out.push(snapshot.clients[index].clone());
                }
            }
        }
        for index in 0..snapshot.clients.len() {
            let server_id = snapshot.server_ids[index];
            if selected.insert(server_id) {
                out.push(snapshot.clients[index].clone());
            }
        }
        out
    }

    pub(crate) fn apply_topology_update(
        &self,
        topology: ClusterTopology,
    ) -> Result<(), ClientError> {
        let topology = dedupe_topology(topology);
        let mut state = self.state.lock().map_err(publish_state_poisoned)?;
        if topology.version != 0 && topology.version <= state.topology_version {
            return Ok(());
        }
        if state
            .topology
            .as_ref()
            .is_some_and(|current| current.version >= topology.version)
            && topology.version != 0
        {
            return Ok(());
        }
        let stale_filter_ms = state.topology_ttl_ms;
        let topology = if stale_filter_ms > 0 {
            let filtered_topology = topology.with_filtered_stale_servers(stale_filter_ms);
            if filtered_topology.servers.is_empty() {
                topology
            } else {
                filtered_topology
            }
        } else {
            topology
        };
        let topology_version = topology.version;
        let routes = if topology.routes.is_empty() {
            build_ring_routes(&topology.servers)
        } else {
            topology.routes.clone()
        };
        let mut clients = Vec::new();
        let mut server_ids = Vec::new();
        for server in &topology.servers {
            if let Some(transport) = T::from_url(&server.url) {
                let mut client = PublishClient::from_transport(transport);
                if let Some(previous) = state.clients.first() {
                    client.max_response_bytes = previous.max_response_bytes;
                    client.retry_policy = previous.retry_policy;
                }
                clients.push(client);
                server_ids.push(server.id);
            }
        }
        if clients.is_empty() {
            return Err(ClientError::Topology(
                "topology update yielded no reachable key servers".to_string(),
            ));
        }
        if state
            .sticky_server_id
            .is_some_and(|sticky_server_id| !server_ids.contains(&sticky_server_id))
        {
            state.sticky_server_id = None;
            state.sticky_until_ms = 0;
        }
        state.clients = clients;
        state.server_ids = server_ids;
        state.routes = routes;
        state.topology = Some(topology);
        state.topology_version = topology_version;
        state.topology_refreshed_ms = super::helpers::unix_ms_now();
        Ok(())
    }

    pub(crate) fn is_topology_stale(&self, snapshot: &TopologyStateSnapshot<T>) -> bool {
        if snapshot.topology.is_none() || snapshot.topology_refreshed_ms == 0 {
            return false;
        }
        if snapshot.topology_ttl_ms == 0 {
            return false;
        }
        let now = super::helpers::unix_ms_now();
        now.saturating_sub(snapshot.topology_refreshed_ms) > snapshot.topology_ttl_ms
    }

    pub(crate) fn refresh_topology_from_peers(&self, snapshot: &TopologyStateSnapshot<T>) -> bool {
        for topology_url in &snapshot.topology_server_urls {
            let Some(bytes) = T::get_topology(topology_url) else {
                continue;
            };
            match topology::decode_topology(&bytes) {
                Ok(topology) => {
                    if self.apply_topology_update(topology).is_ok() {
                        return true;
                    }
                }
                Err(_) => continue,
            }
        }
        false
    }

    pub(crate) fn discover_topology_if_stale(
        &self,
        snapshot: &TopologyStateSnapshot<T>,
    ) -> Result<TopologyStateSnapshot<T>, ClientError> {
        if snapshot.topology.is_none() {
            return Ok(snapshot.clone());
        }
        if !self.is_topology_stale(snapshot) {
            return Ok(snapshot.clone());
        }
        if self.refresh_topology_from_peers(snapshot) {
            return self.snapshot();
        }
        Ok(snapshot.clone())
    }
}
