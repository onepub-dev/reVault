use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::client::{
    DEFAULT_INITIAL_BACKOFF, DEFAULT_MAX_BACKOFF, DEFAULT_MAX_RESPONSE_BYTES,
    DEFAULT_RETRY_ATTEMPTS,
};
use crate::payload::PayloadType;
use crate::protocol;
use crate::topology::{ClusterTopology, TopologyRoute};

#[derive(Clone, Debug)]
/// Represents publish client.
pub struct PublishClient<T = super::HttpTransport> {
    pub(crate) transport: T,
    pub(crate) max_response_bytes: usize,
    pub(crate) retry_policy: RetryPolicy,
}

#[derive(Clone, Debug)]
/// Represents publish client pool.
pub struct PublishClientPool<T = super::HttpTransport> {
    pub(crate) state: Arc<Mutex<PublishTopologyState<T>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PublishTopologyState<T> {
    pub(crate) clients: Vec<PublishClient<T>>,
    pub(crate) server_ids: Vec<u8>,
    pub(crate) topology: Option<ClusterTopology>,
    pub(crate) routes: Vec<TopologyRoute>,
    pub(crate) topology_version: u64,
    pub(crate) topology_server_urls: Vec<String>,
    pub(crate) topology_ttl_ms: u64,
    pub(crate) topology_refreshed_ms: u64,
    pub(crate) sticky_server_id: Option<u8>,
    pub(crate) sticky_until_ms: u64,
}

#[derive(Clone, Debug)]
/// Represents topology aware response.
pub struct TopologyAwareResponse<R> {
    /// Represents the value carried by this record case.
    pub value: R,
    /// Represents the topology carried by this record case.
    pub topology: Option<ClusterTopology>,
}

#[derive(Clone, Debug)]
pub(crate) struct TopologyStateSnapshot<T> {
    pub(crate) clients: Vec<PublishClient<T>>,
    pub(crate) server_ids: Vec<u8>,
    pub(crate) routes: Vec<TopologyRoute>,
    pub(crate) topology: Option<ClusterTopology>,
    pub(crate) topology_version: u64,
    pub(crate) topology_server_urls: Vec<String>,
    pub(crate) topology_ttl_ms: u64,
    pub(crate) topology_refreshed_ms: u64,
    pub(crate) sticky_server_id: Option<u8>,
    pub(crate) sticky_until_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Represents sticky publish server.
pub struct StickyPublishServer {
    /// Represents the server id carried by this record case.
    pub server_id: u8,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
}

impl<T: Clone> PublishTopologyState<T> {
    pub(crate) fn snapshot(&self) -> TopologyStateSnapshot<T> {
        TopologyStateSnapshot {
            clients: self.clients.clone(),
            server_ids: self.server_ids.clone(),
            routes: self.routes.clone(),
            topology: self.topology.clone(),
            topology_version: self.topology_version,
            topology_server_urls: self.topology_server_urls.clone(),
            topology_ttl_ms: self.topology_ttl_ms,
            topology_refreshed_ms: self.topology_refreshed_ms,
            sticky_server_id: self.sticky_server_id,
            sticky_until_ms: self.sticky_until_ms,
        }
    }
}

#[derive(Clone, Debug)]
/// Represents http transport.
pub struct HttpTransport {
    pub(crate) endpoint: Endpoint,
    pub(crate) timeout: Duration,
}

#[derive(Clone, Debug)]
pub(crate) struct Endpoint {
    pub(crate) scheme: Scheme,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) path: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RetryPolicy {
    pub(crate) attempts: usize,
    pub(crate) initial_backoff: Duration,
    pub(crate) max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            attempts: DEFAULT_RETRY_ATTEMPTS,
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scheme {
    Http,
    Https,
}

#[derive(Clone, Debug)]
/// Represents publish result.
pub struct PublishResult {
    /// Represents the publish code carried by this record case.
    pub publish_code: String,
    /// Represents the delete token carried by this record case.
    pub delete_token: Vec<u8>,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
    /// Represents the max receives carried by this record case.
    pub max_receives: u16,
    /// Represents the verification url carried by this record case.
    pub verification_url: Option<String>,
}

#[derive(Clone, Debug)]
/// Represents received publish.
pub struct ReceivedPublish {
    /// Represents the payload carried by this record case.
    pub payload: Vec<u8>,
    /// Represents the payload type carried by this record case.
    pub payload_type: PayloadType,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
    /// Represents the remaining receives carried by this record case.
    pub remaining_receives: u16,
    /// Represents the email verification carried by this record case.
    pub email_verification: Option<protocol::EmailVerification>,
}

/// Represents transport.
pub trait Transport: Clone {
    /// Returns the post binary.
    fn post_binary(&self, body: &[u8]) -> Result<Vec<u8>, super::ClientError>;

    /// Returns topology.
    fn get_topology(_url: &str) -> Option<Vec<u8>> {
        None
    }

    /// Returns the from url.
    fn from_url(_url: &str) -> Option<Self> {
        None
    }

    /// Returns the post binary with topology.
    fn post_binary_with_topology(
        &self,
        body: &[u8],
        topology_version: Option<u64>,
    ) -> Result<Vec<u8>, super::ClientError> {
        if topology_version.is_some() {
            self.post_binary_with_header(body, topology_version)
        } else {
            self.post_binary(body)
        }
    }

    /// Returns the post binary with header.
    fn post_binary_with_header(
        &self,
        body: &[u8],
        topology_version: Option<u64>,
    ) -> Result<Vec<u8>, super::ClientError> {
        let _ = topology_version;
        self.post_binary(body)
    }
}

#[derive(Clone, Copy, Debug)]
/// Represents contact publish.
pub struct ContactPublish<'a> {
    /// Represents the profile carried by this record case.
    pub profile: &'a str,
    /// Represents the public key carried by this record case.
    pub public_key: &'a [u8],
    /// Represents the signing public key carried by this record case.
    pub signing_public_key: &'a [u8],
    /// Represents the fingerprint carried by this record case.
    pub fingerprint: &'a [u8],
    /// Represents the publish nonce carried by this record case.
    pub publish_nonce: &'a [u8],
    /// Represents the created at unix ms carried by this record case.
    pub created_at_unix_ms: u64,
    /// Represents the expires at unix ms carried by this record case.
    pub expires_at_unix_ms: u64,
    /// Represents the verification email carried by this record case.
    pub verification_email: Option<&'a str>,
}

impl<T> PublishClient<T> {
    pub(crate) fn new_inner(transport: T) -> Self {
        Self {
            transport,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            retry_policy: RetryPolicy::default(),
        }
    }
}
