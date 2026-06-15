mod error;
mod helpers;
mod http;
mod pool;
mod pool_topology;
mod single;
mod types;

pub use error::ClientError;
pub use types::{
    ContactPublish, HttpTransport, PublishClient, PublishClientPool, PublishResult,
    ReceivedPublish, StickyPublishServer, TopologyAwareResponse, Transport,
};
pub(crate) use types::{
    Endpoint, PublishTopologyState, RetryPolicy, Scheme, TopologyStateSnapshot,
};

pub(crate) const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024;
pub(crate) const DEFAULT_RETRY_ATTEMPTS: usize = 3;
pub(crate) const DEFAULT_INITIAL_BACKOFF: std::time::Duration =
    std::time::Duration::from_millis(10);
pub(crate) const DEFAULT_MAX_BACKOFF: std::time::Duration = std::time::Duration::from_millis(100);
pub(crate) const DEFAULT_TOPOLOGY_TTL_MS: u64 = 60_000;
pub(crate) const DEFAULT_STICKY_SERVER_TTL_MS: u64 = 24 * 60 * 60 * 1_000;
