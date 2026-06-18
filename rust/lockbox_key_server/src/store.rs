use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::server_log::log_server_event;
use getrandom::getrandom;
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport as MailTransport};
use sha2::{Digest, Sha256};

use lockbox_publish_protocol::client::{HttpTransport, Transport};
use lockbox_publish_protocol::payload;
use lockbox_publish_protocol::protocol::{self, Operation, Reader, Status};
use lockbox_publish_protocol::ClientError;
use lockbox_publish_protocol::{
    build_ring_routes, decode_topology, decode_topology_registration, encode_replication_request,
    encode_topology, encode_topology_registration, publish_code_locator,
    publish_code_server_id_char, sign_replication_event, ClusterTopology, ReplicationEvent,
    ReplicationEventKind, ReplicationRequest, ServerStatus, TopologyRegistration, TopologyRoute,
    TopologyServer,
};

const RECORD_MAGIC: &[u8; 4] = b"LBSF";
const RECORD_HEADER_LEN: usize = 20;
const KIND_PUT: u16 = 1;
const KIND_TOMBSTONE: u16 = 2;
const KIND_RECEIVE_COUNT: u16 = 3;
const DEFAULT_SECRET_LEN: usize = 32;
const HASH_LEN: usize = 16;
const BUCKET_COUNT: usize = 4096;
const BUCKET_RECORD_LEN: usize = 80;
const BUCKET_PUT: u8 = 1;
const BUCKET_TOMBSTONE: u8 = 2;
const BUCKET_RECEIVE_COUNT: u8 = 3;
const OUTBOX_MAGIC: &[u8; 4] = b"LBSO";
const OUTBOX_HEADER_LEN: usize = 16;
const OUTBOX_EVENT: u16 = 1;
const OUTBOX_ACK: u16 = 2;
const REPLICATION_STATE_MAGIC: &[u8; 8] = b"LBSR2\0\0\0";
const REPLICATION_STATE_PERSIST_INTERVAL: usize = 1024;
const SHARE_CODE_BODY_DIGITS: usize = 12;
const TOPOLOGY_HEARTBEAT_INTERVAL_MS: u64 = 30_000;
const DEFAULT_TOPOLOGY_STALE_MS: u64 = 90_000;
const DEFAULT_SMTP_TIMEOUT_SECONDS: u64 = 30;
const EMAIL_QUEUE_CAPACITY: usize = 256;
const STATUS_CACHE_TTL: Duration = Duration::from_secs(1);
const VERIFICATION_ABUSE_BLOCK_TTL: Duration = Duration::from_secs(24 * 60 * 60);

type RecordHash = [u8; HASH_LEN];
type ExpiryBucket = (u64, Vec<(RecordHash, String)>);
type PendingOutbox = VecDeque<(u64, Vec<u8>)>;

#[derive(Debug, Default)]
struct ReplicationState {
    origins: HashMap<u8, ReplicationOriginState>,
    accepted_since_persist: usize,
}

#[derive(Debug, Default)]
struct ReplicationOriginState {
    epoch: u64,
    contiguous_sequence: u64,
    gaps: HashSet<u64>,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub state_dir: PathBuf,
    pub server_id: u8,
    pub cluster_id: String,
    pub public_url: Option<String>,
    pub topology_version: u64,
    pub topology_servers: Vec<TopologyServer>,
    pub topology_routes: Vec<TopologyRoute>,
    pub replication_token: Option<String>,
    pub replication_peer_urls: Vec<String>,
    pub origin_epoch: u64,
    pub promoted_owner_ids: Vec<u8>,
    pub max_payload_bytes: usize,
    pub verification_ttl: Duration,
    pub default_receive_ttl: Duration,
    pub max_receive_ttl: Duration,
    pub shard_count: usize,
    pub developer_mode: bool,
    pub benchmark_requests: usize,
    pub benchmark_payload_bytes: usize,
    pub benchmark_concurrency: usize,
    pub benchmark_preload_published_payloads: usize,
    pub max_receives_per_publish: u16,
    pub compact_min_bytes: u64,
    pub index_cache_entries: usize,
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_tls: SmtpTlsMode,
    pub smtp_timeout: Duration,
    pub verification_email_subject: String,
    pub verification_email_template: String,
    pub verification_email_rate_limit_per_hour: u32,
    pub verification_email_ip_rate_limit_per_hour: u32,
    pub topology_token: Option<String>,
    pub topology_stale_after_ms: u64,
    pub topology_heartbeat_interval_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmtpTlsMode {
    StartTls,
    Tls,
    None,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8089".to_string(),
            state_dir: PathBuf::from("/var/lib/lockbox-key-server"),
            server_id: 0,
            cluster_id: "default".to_string(),
            public_url: None,
            topology_version: 1,
            topology_servers: Vec::new(),
            topology_routes: Vec::new(),
            replication_token: None,
            replication_peer_urls: Vec::new(),
            origin_epoch: unix_ms(SystemTime::now()),
            promoted_owner_ids: Vec::new(),
            max_payload_bytes: 8 * 1024,
            verification_ttl: Duration::from_secs(30 * 60),
            default_receive_ttl: Duration::from_secs(2 * 60 * 60),
            max_receive_ttl: Duration::from_secs(2 * 60 * 60),
            shard_count: 16,
            developer_mode: false,
            benchmark_requests: 50_000,
            benchmark_payload_bytes: 512,
            benchmark_concurrency: 0,
            benchmark_preload_published_payloads: 0,
            max_receives_per_publish: 8,
            compact_min_bytes: 64 * 1024 * 1024,
            index_cache_entries: 65_536,
            rate_limit_per_minute: 120,
            rate_limit_burst: 40,
            smtp_host: None,
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            smtp_from: None,
            smtp_tls: SmtpTlsMode::StartTls,
            smtp_timeout: Duration::from_secs(DEFAULT_SMTP_TIMEOUT_SECONDS),
            verification_email_subject: "Verify your reVault publish".to_string(),
            verification_email_template:
                "Verify {email} for this reVault publish:\n\n{verification_url}\n\nThis link expires in 30 minutes.".to_string(),
            verification_email_rate_limit_per_hour: 5,
            verification_email_ip_rate_limit_per_hour: 30,
            topology_token: None,
            topology_stale_after_ms: DEFAULT_TOPOLOGY_STALE_MS,
            topology_heartbeat_interval_ms: TOPOLOGY_HEARTBEAT_INTERVAL_MS,
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Protocol(protocol::ProtocolError),
    PayloadTooLarge,
    NotFound,
    Expired,
    Exhausted,
    DeleteTokenInvalid,
    PayloadInvalid(String),
    Config(String),
    ReplicationUnauthorized,
    RateLimited,
    EmailUnverified,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Protocol(err) => write!(f, "{err}"),
            Self::PayloadTooLarge => write!(f, "payload too large"),
            Self::NotFound => write!(f, "publish not found"),
            Self::Expired => write!(f, "publish expired"),
            Self::Exhausted => write!(f, "publish exhausted"),
            Self::DeleteTokenInvalid => write!(f, "delete token invalid"),
            Self::PayloadInvalid(err) => write!(f, "payload invalid: {err}"),
            Self::Config(err) => write!(f, "{err}"),
            Self::ReplicationUnauthorized => write!(f, "replication unauthorized"),
            Self::RateLimited => write!(f, "rate limited"),
            Self::EmailUnverified => write!(f, "publisher email is not verified"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn lock_store<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>, StoreError> {
    mutex
        .lock()
        .map_err(|_| StoreError::Config(format!("{name} lock was poisoned")))
}

impl From<protocol::ProtocolError> for StoreError {
    fn from(value: protocol::ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

fn store_error_from_client_error(err: ClientError) -> StoreError {
    match err {
        ClientError::Protocol(err) => StoreError::Protocol(err),
        ClientError::Payload(err) => StoreError::PayloadInvalid(err.to_string()),
        ClientError::Topology(err) => StoreError::PayloadInvalid(err),
        ClientError::Io(err) => StoreError::Io(err),
        ClientError::Url(err) => StoreError::Config(err),
        ClientError::Http(err) => StoreError::Config(err),
        ClientError::Replication(err) => StoreError::Config(err),
        ClientError::Server { message, .. } => StoreError::Config(message),
        ClientError::UnexpectedOperation { expected, actual } => {
            StoreError::Config(format!("unexpected operation {expected:?} != {actual:?}"))
        }
    }
}

#[derive(Clone)]
struct PublishEntry {
    publish_code: String,
    delete_token_hash: RecordHash,
    contact_email: Option<String>,
    payload_offset: u64,
    payload_len: u32,
    expires_at_ms: u64,
    receive_ttl_ms: u64,
    email_verified_at_ms: u64,
    max_receives: u16,
    receives: u16,
}

struct ReplicaPut<'a> {
    publish_code: &'a str,
    delete_token_hash: &'a [u8],
    payload: &'a [u8],
    contact_email: Option<&'a str>,
    expires_at_ms: u64,
    receive_ttl_ms: u64,
    email_verified_at_ms: u64,
    max_receives: u16,
    receives: u16,
}

struct ReplicationQueue<'a> {
    sequence: &'a mut u64,
    sequence_path: &'a Path,
    outbox_path: &'a Path,
    pending: &'a mut PendingOutbox,
    origin_server_id: u8,
    origin_epoch: u64,
    token: &'a [u8],
}

struct Shard {
    path: PathBuf,
    file: Mutex<File>,
    index: Mutex<HashMap<RecordHash, PublishEntry>>,
    expiry_buckets: Mutex<VecDeque<ExpiryBucket>>,
}

pub struct PublishStore {
    config: ServerConfig,
    auto_routes: bool,
    secret: [u8; 32],
    bucket_dir: PathBuf,
    shards: Vec<Shard>,
    verifications: Mutex<HashMap<String, VerificationEntry>>,
    email_rate_limits: Mutex<EmailRateLimits>,
    created: AtomicU64,
    received: AtomicU64,
    email_tx: Option<mpsc::SyncSender<VerificationEmailJob>>,
    rate_limit_blocks: Mutex<HashMap<IpAddr, u64>>,
    verification_email_blocks: Mutex<HashMap<String, u64>>,
    status_cache: Mutex<Option<StatusCache>>,
    deleted: AtomicU64,
    expired: AtomicU64,
    misses: AtomicU64,
    live: AtomicUsize,
    replication_state: Mutex<ReplicationState>,
    replication_state_path: PathBuf,
    replication_tx: Option<mpsc::SyncSender<ReplicationEventKind>>,
    replication_outbox_path: PathBuf,
    replication_sequence_path: PathBuf,
    topology: Mutex<ClusterTopology>,
}

pub struct CreatedPublish {
    pub publish_code: String,
    pub delete_token: Vec<u8>,
    pub expires_at_ms: u64,
    pub max_receives: u16,
    pub verification_url: Option<String>,
}

pub struct ReceivedPublish {
    pub payload: Vec<u8>,
    pub expires_at_ms: u64,
    pub remaining_receives: u16,
    pub email_verification: Option<protocol::EmailVerification>,
}

#[derive(Clone, Debug)]
struct VerificationEmailJob {
    email: String,
    subject: String,
    body: String,
}

struct StatusCache {
    cached_at: Instant,
    document: lockbox_publish_protocol::KeyServerStatus,
}

struct VerificationEntry {
    email: String,
    token_hash: RecordHash,
    expires_at_ms: u64,
}

#[derive(Default)]
struct EmailRateLimits {
    by_email: HashMap<String, VecDeque<u64>>,
    by_ip: HashMap<IpAddr, VecDeque<u64>>,
}

#[derive(Clone, Debug)]
pub struct VerificationPage {
    pub success: bool,
    pub title: String,
    pub message: String,
    pub email: Option<String>,
}

fn spawn_verification_email_worker(
    config: &ServerConfig,
) -> Option<mpsc::SyncSender<VerificationEmailJob>> {
    config.smtp_host.as_ref()?;
    let config = config.clone();
    let (tx, rx) = mpsc::sync_channel::<VerificationEmailJob>(EMAIL_QUEUE_CAPACITY);
    thread::Builder::new()
        .name("publish-email".to_string())
        .stack_size(256 * 1024)
        .spawn(move || {
            for job in rx {
                if let Err(err) = PublishStore::send_verification_email_smtp(
                    &config,
                    &job.email,
                    &job.subject,
                    &job.body,
                ) {
                    log_server_event(format!("verification email send failed: {err}"));
                }
            }
        })
        .ok()?;
    Some(tx)
}

impl PublishStore {
    fn encode_response_with_topology(
        &self,
        operation: Operation,
        status: Status,
        payload: &[u8],
    ) -> Vec<u8> {
        let base = protocol::encode_response(operation, status, payload);
        let topology = self.topology();
        match encode_topology(&topology) {
            Ok(bytes) => protocol::encode_response_with_tail(operation, status, payload, &bytes),
            Err(_) => base,
        }
    }

    fn encode_store_error_with_topology(&self, operation: Operation, err: StoreError) -> Vec<u8> {
        let status = match err {
            StoreError::PayloadTooLarge => Status::PayloadTooLarge,
            StoreError::NotFound => Status::PublishNotFound,
            StoreError::Expired => Status::PublishExpired,
            StoreError::Exhausted => Status::PublishExhausted,
            StoreError::DeleteTokenInvalid => Status::DeleteTokenInvalid,
            StoreError::PayloadInvalid(_) => Status::MalformedRequest,
            StoreError::Protocol(_) => Status::MalformedRequest,
            StoreError::Config(_) => Status::StoreUnavailable,
            StoreError::ReplicationUnauthorized => Status::ReplicationUnauthorized,
            StoreError::RateLimited => Status::RateLimited,
            StoreError::EmailUnverified => Status::EmailUnverified,
            StoreError::Io(_) => Status::StoreUnavailable,
        };
        let mut payload = Vec::new();
        protocol::put_u16(&mut payload, protocol::MESSAGE_VERSION);
        protocol::put_u16(&mut payload, status as u16);
        protocol::put_string(&mut payload, &err.to_string());
        self.encode_response_with_topology(operation, status, &payload)
    }
    pub fn open(mut config: ServerConfig) -> Result<Self, StoreError> {
        const MAX_SERVER_ID: u8 = 35;
        if config.developer_mode
            && config.state_dir.as_path() == Path::new("/var/lib/lockbox-key-server")
        {
            config.state_dir = std::env::temp_dir().join("lockbox-key-server-dev");
        }
        if config.server_id > MAX_SERVER_ID {
            return Err(StoreError::Config(
                "server id must be an index 0..35 (0..9, a..z)".to_string(),
            ));
        }
        for promoted_owner_id in &config.promoted_owner_ids {
            if *promoted_owner_id > MAX_SERVER_ID {
                return Err(StoreError::Config(
                    "promoted owner id must be an index 0..35 (0..9, a..z)".to_string(),
                ));
            }
        }
        for topology_server in &config.topology_servers {
            if topology_server.id > MAX_SERVER_ID {
                return Err(StoreError::Config(
                    "topology server id must be an index 0..35 (0..9, a..z)".to_string(),
                ));
            }
        }
        for topology_route in &config.topology_routes {
            if topology_route.owner_id > MAX_SERVER_ID
                || topology_route.primary_id > MAX_SERVER_ID
                || topology_route
                    .failover_ids
                    .iter()
                    .any(|id| *id > MAX_SERVER_ID)
            {
                return Err(StoreError::Config(
                    "topology route id must be an index 0..35 (0..9, a..z)".to_string(),
                ));
            }
        }
        fs::create_dir_all(&config.state_dir)?;
        let auto_routes = config.topology_routes.is_empty();
        let bucket_dir = config.state_dir.join("index");
        fs::create_dir_all(&bucket_dir)?;
        let replication_state_path = config.state_dir.join("replication-state.bin");
        let replication_state = load_replication_state(&replication_state_path)?;
        let replication_outbox_path = config.state_dir.join("replication-outbox.bin");
        let replication_sequence_path = config.state_dir.join("replication-origin-sequence");
        let replication_tx = start_replication_worker(
            &config,
            &replication_outbox_path,
            &replication_sequence_path,
        );
        let secret = load_or_create_secret(&config.state_dir)?;
        let shard_count = config.shard_count.max(1);
        let cache_per_shard = config.index_cache_entries / shard_count;
        let mut shards = Vec::with_capacity(shard_count);
        let mut live = 0;
        for shard_id in 0..shard_count {
            let path = config
                .state_dir
                .join(format!("published-payloads-{shard_id:03}.seg"));
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .append(true)
                .open(&path)?;
            let mut index = replay(&mut file)?;
            live += index.len();
            if cache_per_shard > 0 && index.len() > cache_per_shard {
                index = index.into_iter().take(cache_per_shard).collect();
            }
            shards.push(Shard {
                path,
                file: Mutex::new(file),
                index: Mutex::new(index),
                expiry_buckets: Mutex::new(VecDeque::new()),
            });
        }
        let email_tx = spawn_verification_email_worker(&config);
        let topology = Self::build_initial_topology(&config);
        Ok(Self {
            config,
            auto_routes,
            secret,
            bucket_dir,
            shards,
            verifications: Mutex::new(HashMap::new()),
            email_rate_limits: Mutex::new(EmailRateLimits::default()),
            created: AtomicU64::new(0),
            received: AtomicU64::new(0),
            email_tx,
            rate_limit_blocks: Mutex::new(HashMap::new()),
            verification_email_blocks: Mutex::new(HashMap::new()),
            status_cache: Mutex::new(None),
            deleted: AtomicU64::new(0),
            expired: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            live: AtomicUsize::new(live),
            replication_state: Mutex::new(replication_state),
            replication_state_path,
            replication_tx,
            replication_outbox_path,
            replication_sequence_path,
            topology: Mutex::new(topology),
        })
    }

    pub fn handle(&self, operation: Operation, payload: &[u8]) -> Vec<u8> {
        self.handle_with_peer(operation, payload, None)
    }

    pub fn handle_with_peer(
        &self,
        operation: Operation,
        payload: &[u8],
        peer_ip: Option<IpAddr>,
    ) -> Vec<u8> {
        match operation {
            Operation::Publish => self.handle_publish(payload, peer_ip),
            Operation::Receive => self.handle_receive(payload),
            Operation::Delete => self.handle_delete(payload),
            Operation::Replicate => self.handle_replication(payload),
        }
    }

    pub fn max_payload_bytes(&self) -> usize {
        self.config.max_payload_bytes
    }

    pub fn rate_limit_per_minute(&self) -> u32 {
        self.config.rate_limit_per_minute
    }

    pub fn rate_limit_burst(&self) -> u32 {
        self.config.rate_limit_burst
    }

    pub fn is_rate_limit_blocked(&self, peer_ip: Option<IpAddr>) -> bool {
        let Some(peer_ip) = peer_ip else {
            return false;
        };
        let now_ms = unix_ms(SystemTime::now());
        let Ok(mut blocks) = self.rate_limit_blocks.lock() else {
            return true;
        };
        match blocks.get(&peer_ip).copied() {
            Some(expires_at) if expires_at > now_ms => true,
            Some(_) => {
                blocks.remove(&peer_ip);
                false
            }
            None => false,
        }
    }

    pub fn is_verification_email_blocked(&self, email: &str) -> bool {
        let Ok(email) = normalize_verification_email(email) else {
            return true;
        };
        let now_ms = unix_ms(SystemTime::now());
        let Ok(mut blocks) = self.verification_email_blocks.lock() else {
            return true;
        };
        match blocks.get(&email).copied() {
            Some(expires_at) if expires_at > now_ms => true,
            Some(_) => {
                blocks.remove(&email);
                false
            }
            None => false,
        }
    }

    pub fn block_rate_limited_client_until(
        &self,
        peer_ip: IpAddr,
        expires_at_unix_ms: u64,
    ) -> Result<bool, StoreError> {
        let applied = self.apply_rate_limit_block(peer_ip, expires_at_unix_ms)?;
        if applied {
            self.enqueue_replication(ReplicationEventKind::RateLimitBlock {
                client_ip: peer_ip.to_string(),
                expires_at_unix_ms,
            });
        }
        Ok(applied)
    }

    pub fn topology_token_matches(&self, token: &str) -> bool {
        self.config.topology_token.as_deref() == Some(token)
    }

    pub fn start_topology_background(self: &Arc<Self>) {
        if self.config.topology_token.is_none() {
            return;
        }
        if self.config.topology_heartbeat_interval_ms == 0 {
            return;
        }
        let store = Arc::clone(self);
        let interval = Duration::from_millis(self.config.topology_heartbeat_interval_ms.max(1_000));
        thread::spawn(move || loop {
            for peer_url in store.topology_peer_urls() {
                if let Some(register_url) = Self::topology_register_url(&peer_url) {
                    store.send_topology_registration(&register_url);
                }
            }
            thread::sleep(interval);
        });
    }

    fn normalize_routes_for_automembership(&self, topology: &mut ClusterTopology) {
        if self.auto_routes {
            topology.routes = build_ring_routes(&topology.servers);
        }
    }

    pub fn topology(&self) -> ClusterTopology {
        let mut topology = match self.topology.lock() {
            Ok(topology) => topology.clone(),
            Err(_) => Self::build_initial_topology(&self.config),
        };
        let stale_after_ms = self.config.topology_stale_after_ms;
        if stale_after_ms > 0 {
            topology = topology.with_filtered_stale_servers(stale_after_ms);
            if topology.routes.is_empty() {
                topology.routes = build_ring_routes(&topology.servers);
            }
        }
        topology
    }

    pub fn register_topology_server(
        &self,
        registration: TopologyRegistration,
    ) -> Result<ClusterTopology, StoreError> {
        if registration.cluster_id != self.config.cluster_id {
            return Err(StoreError::Config(
                "topology cluster id mismatch".to_string(),
            ));
        }
        if registration.server_url.is_empty() || registration.security_token.is_empty() {
            return Err(StoreError::Config(
                "topology registration missing required fields".to_string(),
            ));
        }
        if self.config.topology_token.as_ref() != Some(&registration.security_token) {
            return Err(StoreError::Config(
                "topology registration token invalid".to_string(),
            ));
        }
        let mut topology = lock_store(&self.topology, "topology")?.clone();
        let now_ms = unix_ms(SystemTime::now());
        if let Some(server) = topology
            .servers
            .iter_mut()
            .find(|server| server.id == registration.server_id)
        {
            if server.url != registration.server_url {
                server.url = registration.server_url;
            }
            server.status = registration.status;
            server.last_seen_ms = Some(now_ms);
        } else {
            topology.servers.push(TopologyServer {
                id: registration.server_id,
                url: registration.server_url,
                status: registration.status,
                last_seen_ms: Some(now_ms),
            });
        }
        if topology.cluster_id != self.config.cluster_id {
            topology.cluster_id = self.config.cluster_id.clone();
        }
        topology.version = topology.version.saturating_add(1);
        topology.validate().map_err(store_error_from_client_error)?;
        topology = topology.with_filtered_stale_servers(self.config.topology_stale_after_ms);
        self.normalize_routes_for_automembership(&mut topology);
        *lock_store(&self.topology, "topology")? = topology.clone();
        Ok(topology)
    }

    pub fn handle_topology_registration(&self, payload: &[u8]) -> Result<Vec<u8>, StoreError> {
        let registration =
            decode_topology_registration(payload).map_err(store_error_from_client_error)?;
        let topology = self.register_topology_server(registration)?;
        encode_topology(&topology).map_err(store_error_from_client_error)
    }

    fn topology_peer_urls(&self) -> Vec<String> {
        self.topology()
            .servers
            .iter()
            .filter(|server| server.id != self.config.server_id)
            .filter(|server| {
                matches!(
                    server.status,
                    ServerStatus::Active | ServerStatus::Promoted | ServerStatus::Standby
                )
            })
            .map(|server| server.url.clone())
            .collect()
    }

    fn send_topology_registration(self: &Arc<Self>, register_url: &str) {
        let token = match &self.config.topology_token {
            Some(token) => token.clone(),
            None => return,
        };
        let topology = self.topology();
        let Some(self_server) = topology
            .servers
            .iter()
            .find(|server| server.id == self.config.server_id)
        else {
            return;
        };
        if matches!(self_server.status, ServerStatus::Disabled) {
            return;
        }
        let registration = TopologyRegistration {
            cluster_id: topology.cluster_id.clone(),
            server_id: self.config.server_id,
            server_url: self_server.url.clone(),
            status: self_server.status.clone(),
            security_token: token.clone(),
        };
        let payload = match encode_topology_registration(&registration) {
            Ok(payload) => payload,
            Err(_) => return,
        };
        let Ok(transport) = HttpTransport::new(register_url) else {
            return;
        };
        let Ok(response) = transport.post_binary_with_server_token(&payload, &token) else {
            return;
        };
        if let Ok(updated) = decode_topology(&response) {
            let _ = self.apply_topology_update(updated);
        }
    }

    fn apply_topology_update(&self, mut topology: ClusterTopology) -> Result<(), StoreError> {
        if topology.with_filtered_stale_servers(0).servers.is_empty() {
            return Err(StoreError::Config("topology has no servers".to_string()));
        }
        if topology.version == 0 {
            topology.version = 1;
        }
        topology = topology.with_filtered_stale_servers(self.config.topology_stale_after_ms);
        self.normalize_routes_for_automembership(&mut topology);
        if topology.servers.is_empty() {
            return Err(StoreError::Config(
                "topology has no healthy servers".to_string(),
            ));
        }
        topology.version = topology.version.max(1);
        let version = lock_store(&self.topology, "topology")?.version;
        if topology.version < version {
            return Ok(());
        }
        *lock_store(&self.topology, "topology")? = topology;
        Ok(())
    }

    fn build_initial_topology(config: &ServerConfig) -> ClusterTopology {
        let servers = if config.topology_servers.is_empty() {
            vec![TopologyServer {
                id: config.server_id,
                url: config
                    .public_url
                    .clone()
                    .unwrap_or_else(|| format!("http://{}/v1/publish", config.bind_addr)),
                status: ServerStatus::Active,
                last_seen_ms: None,
            }]
        } else {
            config.topology_servers.clone()
        };
        let mut routes = if config.topology_routes.is_empty() {
            build_ring_routes(&servers)
        } else {
            config.topology_routes.clone()
        };
        if routes.is_empty() {
            routes = vec![TopologyRoute {
                owner_id: config.server_id,
                primary_id: config.server_id,
                failover_ids: vec![config.server_id],
            }];
        }
        ClusterTopology {
            cluster_id: config.cluster_id.clone(),
            version: config.topology_version,
            servers,
            routes,
        }
    }

    fn topology_register_url(server_url: &str) -> Option<String> {
        let trimmed = server_url.trim().trim_end_matches('/');
        let base = trimmed
            .split("/v1/")
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(trimmed);
        Some(format!("{base}/v1/topology/register"))
    }

    fn public_publish_url(&self) -> String {
        self.config.public_url.clone().unwrap_or_else(|| {
            let authority = self
                .config
                .bind_addr
                .strip_prefix("0.0.0.0:")
                .map(|port| format!("127.0.0.1:{port}"))
                .unwrap_or_else(|| self.config.bind_addr.clone());
            format!("http://{authority}/v1/publish")
        })
    }

    fn handle_publish(&self, payload: &[u8], peer_ip: Option<IpAddr>) -> Vec<u8> {
        match self.create_from_payload_with_peer(payload, peer_ip) {
            Ok(created) => {
                let mut body = Vec::new();
                protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
                protocol::put_string(&mut body, &created.publish_code);
                protocol::put_bytes(&mut body, &created.delete_token);
                protocol::put_u64(&mut body, created.expires_at_ms);
                protocol::put_u16(&mut body, created.max_receives);
                if let Some(verification_url) = &created.verification_url {
                    protocol::put_string(&mut body, verification_url);
                }
                self.encode_response_with_topology(Operation::Publish, Status::Success, &body)
            }
            Err(err) => self.encode_store_error_with_topology(Operation::Publish, err),
        }
    }

    fn handle_receive(&self, payload: &[u8]) -> Vec<u8> {
        let result = (|| {
            let mut reader = Reader::new(payload);
            reader.message_version()?;
            let publish_code = reader.string()?;
            self.receive_by_lookup(&publish_code)
        })();
        match result {
            Ok(received) => {
                let mut body = Vec::new();
                protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
                protocol::put_bytes(&mut body, &received.payload);
                protocol::put_u64(&mut body, received.expires_at_ms);
                protocol::put_u16(&mut body, received.remaining_receives);
                if let Some(verification) = &received.email_verification {
                    protocol::put_string(&mut body, &verification.email);
                    body.push(u8::from(verification.verified));
                    protocol::put_u64(&mut body, verification.verified_at_unix_ms);
                    protocol::put_bytes(&mut body, &verification.attestation);
                }
                self.encode_response_with_topology(Operation::Receive, Status::Success, &body)
            }
            Err(err) => self.encode_store_error_with_topology(Operation::Receive, err),
        }
    }

    fn handle_delete(&self, payload: &[u8]) -> Vec<u8> {
        let result = (|| {
            let mut reader = Reader::new(payload);
            reader.message_version()?;
            let publish_code = reader.string()?;
            let token = reader.bytes()?;
            self.delete(&publish_code, &token)
        })();
        match result {
            Ok(deleted) => {
                let mut body = Vec::new();
                protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
                body.push(u8::from(deleted));
                self.encode_response_with_topology(Operation::Delete, Status::Success, &body)
            }
            Err(err) => self.encode_store_error_with_topology(Operation::Delete, err),
        }
    }

    fn handle_replication(&self, payload: &[u8]) -> Vec<u8> {
        match self.apply_replication_payload(payload) {
            Ok(_) => self.encode_response_with_topology(Operation::Replicate, Status::Success, &[]),
            Err(StoreError::ReplicationUnauthorized) => self.encode_store_error_with_topology(
                Operation::Replicate,
                StoreError::ReplicationUnauthorized,
            ),
            Err(err) => self.encode_store_error_with_topology(Operation::Replicate, err),
        }
    }

    pub fn create_from_payload(&self, payload: &[u8]) -> Result<CreatedPublish, StoreError> {
        self.create_from_payload_with_peer(payload, None)
    }

    pub fn create_from_payload_with_peer(
        &self,
        payload: &[u8],
        peer_ip: Option<IpAddr>,
    ) -> Result<CreatedPublish, StoreError> {
        let mut reader = Reader::new(payload);
        reader.message_version()?;
        let ttl_seconds = reader.u32()?;
        let requested_receives = reader.u16()?;
        let max_receives = if requested_receives == 0 {
            1
        } else {
            requested_receives.min(self.config.max_receives_per_publish.max(1))
        };
        let publish_payload = reader.bytes()?;
        let verification_email = if reader.is_done() {
            return Err(StoreError::PayloadInvalid(
                "publisher email is required".to_string(),
            ));
        } else {
            let email = reader.string()?;
            normalize_verification_email(&email)?
        };
        if publish_payload.len() > self.config.max_payload_bytes {
            return Err(StoreError::PayloadTooLarge);
        }
        self.check_verification_blocks(&verification_email, peer_ip)?;
        payload::validate_payload(&publish_payload)
            .map_err(|err| StoreError::PayloadInvalid(err.to_string()))?;
        self.check_email_rate_limit(&verification_email, peer_ip)?;
        let receive_ttl = if ttl_seconds == 0 {
            self.config.default_receive_ttl
        } else {
            Duration::from_secs(ttl_seconds as u64).min(self.config.max_receive_ttl)
        };
        let verification_expires_at_ms = unix_ms(SystemTime::now() + self.config.verification_ttl);
        let receive_ttl_ms = receive_ttl.as_millis() as u64;
        let publish_code = self.generate_unique_code()?;
        let mut delete_token = vec![0_u8; DEFAULT_SECRET_LEN];
        getrandom(&mut delete_token)
            .map_err(|err| StoreError::Io(std::io::Error::other(err.to_string())))?;
        let code_hash = self.code_hash(&publish_code);
        let delete_token_hash = self.delete_token_hash(&delete_token);
        let mut entry = PublishEntry {
            publish_code: publish_code.clone(),
            delete_token_hash,
            contact_email: Some(verification_email.clone()),
            payload_offset: 0,
            payload_len: publish_payload.len() as u32,
            expires_at_ms: verification_expires_at_ms,
            receive_ttl_ms,
            email_verified_at_ms: 0,
            max_receives,
            receives: 0,
        };
        let shard_id = self.shard_for(&code_hash);
        let shard = &self.shards[shard_id];
        let mut index = lock_store(&shard.index, "shard index")?;
        let mut file = lock_store(&shard.file, "shard file")?;
        let (payload_offset, payload_len) =
            append_put(&mut file, &code_hash, &entry, &publish_payload)?;
        entry.payload_offset = payload_offset;
        entry.payload_len = payload_len;
        self.append_bucket_put(&code_hash, &entry)?;
        let contact_email = entry.contact_email.clone();
        if index.len() < self.config.index_cache_entries / self.shards.len().max(1) {
            index.insert(code_hash, entry);
        }
        lock_store(&shard.expiry_buckets, "expiry buckets")?.push_back((
            verification_expires_at_ms,
            vec![(code_hash, publish_code.clone())],
        ));
        self.created.fetch_add(1, Ordering::Relaxed);
        self.live.fetch_add(1, Ordering::Relaxed);
        self.enqueue_replication(ReplicationEventKind::PutPublish {
            publish_code: publish_code.clone(),
            delete_token_hash: delete_token_hash.to_vec(),
            payload: publish_payload,
            contact_email,
            expires_at_unix_ms: verification_expires_at_ms,
            receive_ttl_ms,
            email_verified_at_unix_ms: 0,
            max_receives,
            receives: 0,
        });
        let verification_url = Some(self.create_verification(
            &publish_code,
            &verification_email,
            verification_expires_at_ms,
        )?);
        Ok(CreatedPublish {
            publish_code,
            delete_token,
            expires_at_ms: verification_expires_at_ms,
            max_receives,
            verification_url,
        })
    }

    fn check_email_rate_limit(
        &self,
        email: &str,
        peer_ip: Option<IpAddr>,
    ) -> Result<(), StoreError> {
        let email_limit = self.config.verification_email_rate_limit_per_hour as usize;
        let ip_limit = self.config.verification_email_ip_rate_limit_per_hour as usize;
        if email_limit == 0 && (ip_limit == 0 || peer_ip.is_none()) {
            return Ok(());
        }

        let now = unix_ms(SystemTime::now());
        let cutoff = now.saturating_sub(Duration::from_secs(60 * 60).as_millis() as u64);
        let mut limits = lock_store(&self.email_rate_limits, "email rate limits")?;

        if email_limit != 0 {
            let bucket = limits.by_email.entry(email.to_string()).or_default();
            prune_rate_bucket(bucket, cutoff);
            if bucket.len() >= email_limit {
                self.block_verification_email_until(
                    email,
                    verification_abuse_block_expires_at_ms(),
                )?;
                return Err(StoreError::RateLimited);
            }
        }
        if ip_limit != 0 {
            if let Some(ip) = peer_ip {
                let bucket = limits.by_ip.entry(ip).or_default();
                prune_rate_bucket(bucket, cutoff);
                if bucket.len() >= ip_limit {
                    self.block_rate_limited_client_until(
                        ip,
                        verification_abuse_block_expires_at_ms(),
                    )?;
                    return Err(StoreError::RateLimited);
                }
            }
        }

        if email_limit != 0 {
            limits
                .by_email
                .entry(email.to_string())
                .or_default()
                .push_back(now);
        }
        if ip_limit != 0 {
            if let Some(ip) = peer_ip {
                limits.by_ip.entry(ip).or_default().push_back(now);
            }
        }
        Ok(())
    }

    fn check_verification_blocks(
        &self,
        email: &str,
        peer_ip: Option<IpAddr>,
    ) -> Result<(), StoreError> {
        if self.is_verification_email_blocked(email) || self.is_rate_limit_blocked(peer_ip) {
            Err(StoreError::RateLimited)
        } else {
            Ok(())
        }
    }

    fn block_verification_email_until(
        &self,
        email: &str,
        expires_at_unix_ms: u64,
    ) -> Result<bool, StoreError> {
        let now_ms = unix_ms(SystemTime::now());
        if expires_at_unix_ms <= now_ms {
            return Ok(false);
        }
        let email = normalize_verification_email(email)?;
        let mut blocks = lock_store(&self.verification_email_blocks, "verification email blocks")?;
        let current = blocks.get(&email).copied().unwrap_or(0);
        if current >= expires_at_unix_ms {
            return Ok(false);
        }
        blocks.insert(email, expires_at_unix_ms);
        Ok(true)
    }

    fn resolve_publish_lookup(&self, lookup: &str) -> Result<String, StoreError> {
        if lookup.is_empty() {
            return Err(StoreError::NotFound);
        }
        if publish_code_locator(lookup).is_some() {
            if !self.can_serve_publish_code(lookup) {
                return Err(StoreError::NotFound);
            }
            return Ok(lookup.to_string());
        }
        Err(StoreError::NotFound)
    }

    fn create_verification(
        &self,
        publish_code: &str,
        email: &str,
        expires_at_ms: u64,
    ) -> Result<String, StoreError> {
        let mut token = vec![0_u8; DEFAULT_SECRET_LEN];
        getrandom(&mut token)
            .map_err(|err| StoreError::Io(std::io::Error::other(err.to_string())))?;
        let token_hex = hex_encode(&token);
        let token_hash = stable_hash(b"email-verification-token", token_hex.as_bytes());
        lock_store(&self.verifications, "email verifications")?.insert(
            publish_code.to_string(),
            VerificationEntry {
                email: email.to_string(),
                token_hash,
                expires_at_ms,
            },
        );
        let verification_url = format!(
            "{}?code={publish_code}&token={token_hex}",
            self.public_verify_url()
        );
        self.send_verification_email(email, publish_code, &verification_url)?;
        Ok(verification_url)
    }

    fn public_verify_url(&self) -> String {
        let publish_url = self.public_publish_url();
        if let Some(base) = publish_url.strip_suffix("/v1/publish") {
            format!("{base}/v1/verify")
        } else {
            format!("{}/v1/verify", publish_url.trim_end_matches('/'))
        }
    }

    fn verify_email_inner(&self, publish_code: &str, token: &str) -> Result<String, String> {
        if !self.can_serve_publish_code(publish_code) {
            return Err("This server does not own the supplied publish code.".to_string());
        }
        let token_hash = stable_hash(b"email-verification-token", token.as_bytes());
        let now = unix_ms(SystemTime::now());
        let email = {
            let mut verifications = self
                .verifications
                .lock()
                .map_err(|_| "The verification state is unavailable.".to_string())?;
            let Some(entry) = verifications.get(publish_code) else {
                return Err("The verification link is unknown or has expired.".to_string());
            };
            if entry.expires_at_ms <= now {
                verifications.remove(publish_code);
                return Err("The verification link has expired.".to_string());
            }
            if entry.token_hash != token_hash {
                return Err("The verification token is invalid.".to_string());
            }
            let email = entry.email.clone();
            verifications.remove(publish_code);
            email
        };
        self.promote_verified_publish(publish_code, &email, now)
            .map_err(|err| err.to_string())?;
        Ok(email)
    }

    fn promote_verified_publish(
        &self,
        publish_code: &str,
        email: &str,
        verified_at_ms: u64,
    ) -> Result<(), StoreError> {
        let code_hash = self.code_hash(publish_code);
        let shard = &self.shards[self.shard_for(&code_hash)];
        let mut index = lock_store(&shard.index, "shard index")?;
        let cached = index.contains_key(&code_hash);
        let mut entry = match index.get(&code_hash) {
            Some(entry) => entry.clone(),
            None => match self.lookup_bucket(&code_hash)? {
                Some(mut entry) => {
                    entry.publish_code = publish_code.to_string();
                    entry
                }
                None => return Err(StoreError::NotFound),
            },
        };
        if entry.expires_at_ms <= verified_at_ms {
            index.remove(&code_hash);
            return Err(StoreError::Expired);
        }
        if entry.contact_email.as_deref() != Some(email) {
            return Err(StoreError::EmailUnverified);
        }
        if entry.email_verified_at_ms != 0 {
            return Ok(());
        }
        let receive_ttl_ms = if entry.receive_ttl_ms == 0 {
            self.config.default_receive_ttl.as_millis() as u64
        } else {
            entry.receive_ttl_ms
        };
        let mut file = lock_store(&shard.file, "shard file")?;
        let payload = read_payload(&mut file, entry.payload_offset, entry.payload_len)?;
        entry.email_verified_at_ms = verified_at_ms;
        entry.expires_at_ms = verified_at_ms.saturating_add(receive_ttl_ms);
        let (payload_offset, payload_len) = append_put(&mut file, &code_hash, &entry, &payload)?;
        entry.payload_offset = payload_offset;
        entry.payload_len = payload_len;
        self.append_bucket_put(&code_hash, &entry)?;
        if cached || index.len() < self.config.index_cache_entries / self.shards.len().max(1) {
            index.insert(code_hash, entry.clone());
        }
        lock_store(&shard.expiry_buckets, "expiry buckets")?.push_back((
            entry.expires_at_ms,
            vec![(code_hash, publish_code.to_string())],
        ));
        self.enqueue_replication(ReplicationEventKind::PutPublish {
            publish_code: publish_code.to_string(),
            delete_token_hash: entry.delete_token_hash.to_vec(),
            payload,
            contact_email: entry.contact_email.clone(),
            expires_at_unix_ms: entry.expires_at_ms,
            receive_ttl_ms: entry.receive_ttl_ms,
            email_verified_at_unix_ms: entry.email_verified_at_ms,
            max_receives: entry.max_receives,
            receives: entry.receives,
        });
        Ok(())
    }

    fn email_verification_for_receive(
        &self,
        entry: &PublishEntry,
    ) -> Option<protocol::EmailVerification> {
        let email = entry.contact_email.clone()?;
        let verified_at = entry.email_verified_at_ms;
        let verified = verified_at != 0;
        let attestation = if verified {
            self.email_attestation(
                &entry.publish_code,
                &email,
                verified_at,
                entry.expires_at_ms,
            )
        } else {
            Vec::new()
        };
        Some(protocol::EmailVerification {
            email,
            verified,
            verified_at_unix_ms: verified_at,
            attestation,
        })
    }

    fn email_attestation(
        &self,
        publish_code: &str,
        email: &str,
        verified_at_ms: u64,
        expires_at_ms: u64,
    ) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.secret);
        hasher.update(b"email-verification-attestation-v1");
        hasher.update(publish_code.as_bytes());
        hasher.update(email.as_bytes());
        hasher.update(verified_at_ms.to_be_bytes());
        hasher.update(expires_at_ms.to_be_bytes());
        hasher.finalize().to_vec()
    }

    fn send_verification_email(
        &self,
        email: &str,
        publish_code: &str,
        verification_url: &str,
    ) -> Result<(), StoreError> {
        let subject = render_email_template(
            &self.config.verification_email_subject,
            email,
            publish_code,
            verification_url,
        );
        let body = render_email_template(
            &self.config.verification_email_template,
            email,
            publish_code,
            verification_url,
        );
        if let Some(tx) = &self.email_tx {
            return tx
                .try_send(VerificationEmailJob {
                    email: email.to_string(),
                    subject,
                    body,
                })
                .map_err(|err| match err {
                    mpsc::TrySendError::Full(_) => StoreError::RateLimited,
                    mpsc::TrySendError::Disconnected(_) => {
                        StoreError::Config("could not send verification email".to_string())
                    }
                });
        }
        if self.config.developer_mode {
            return Ok(());
        }
        Err(StoreError::Config(
            "could not send verification email".to_string(),
        ))
    }

    fn send_verification_email_smtp(
        config: &ServerConfig,
        email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), StoreError> {
        let host = config
            .smtp_host
            .as_deref()
            .ok_or_else(|| StoreError::Config("could not send verification email".to_string()))?;
        let from = config
            .smtp_from
            .as_deref()
            .or(config.smtp_username.as_deref())
            .ok_or_else(|| StoreError::Config("could not send verification email".to_string()))?;
        let message = Message::builder()
            .from(from.parse::<Mailbox>().map_err(|err| {
                log_server_event(format!("verification email sender address failed: {err}"));
                StoreError::Config("could not send verification email".to_string())
            })?)
            .to(email.parse::<Mailbox>().map_err(|err| {
                StoreError::PayloadInvalid(format!(
                    "verification email is not a valid email address: {err}"
                ))
            })?)
            .subject(subject)
            .singlepart(SinglePart::plain(body.to_string()))
            .map_err(|err| {
                log_server_event(format!("verification email build failed: {err}"));
                StoreError::Config("could not send verification email".to_string())
            })?;
        let mut builder = match config.smtp_tls {
            SmtpTlsMode::StartTls => SmtpTransport::starttls_relay(host).map_err(|err| {
                log_server_event(format!("smtp setup failed: {err}"));
                StoreError::Config("could not send verification email".to_string())
            })?,
            SmtpTlsMode::Tls => SmtpTransport::relay(host).map_err(|err| {
                log_server_event(format!("smtp setup failed: {err}"));
                StoreError::Config("could not send verification email".to_string())
            })?,
            SmtpTlsMode::None => SmtpTransport::builder_dangerous(host),
        }
        .port(config.smtp_port)
        .timeout(Some(config.smtp_timeout));
        if let (Some(username), Some(password)) = (&config.smtp_username, &config.smtp_password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        }
        builder.build().send(&message).map_err(|err| {
            log_server_event(format!("verification email send failed: {err}"));
            StoreError::Config("could not send verification email".to_string())
        })?;
        Ok(())
    }

    pub fn receive(&self, publish_code: &str) -> Result<ReceivedPublish, StoreError> {
        if !self.can_serve_publish_code(publish_code) {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return Err(StoreError::NotFound);
        }
        let code_hash = self.code_hash(publish_code);
        let shard = &self.shards[self.shard_for(&code_hash)];
        let mut index = lock_store(&shard.index, "shard index")?;
        let mut cached = true;
        let mut entry = match index.get(&code_hash) {
            Some(entry) => entry.clone(),
            None => {
                cached = false;
                match self.lookup_bucket(&code_hash)? {
                    Some(entry) => entry,
                    None => {
                        self.misses.fetch_add(1, Ordering::Relaxed);
                        return Err(StoreError::NotFound);
                    }
                }
            }
        };
        if entry.expires_at_ms <= unix_ms(SystemTime::now()) {
            index.remove(&code_hash);
            let mut file = lock_store(&shard.file, "shard file")?;
            append_tombstone(&mut file, &code_hash)?;
            self.append_bucket_tombstone(&code_hash)?;
            self.enqueue_replication(ReplicationEventKind::Tombstone {
                publish_code: publish_code.to_string(),
            });
            self.expired.fetch_add(1, Ordering::Relaxed);
            self.live.fetch_sub(1, Ordering::Relaxed);
            return Err(StoreError::Expired);
        }
        if entry.receives >= entry.max_receives {
            index.remove(&code_hash);
            let mut file = lock_store(&shard.file, "shard file")?;
            append_tombstone(&mut file, &code_hash)?;
            self.append_bucket_tombstone(&code_hash)?;
            self.enqueue_replication(ReplicationEventKind::Tombstone {
                publish_code: publish_code.to_string(),
            });
            self.live.fetch_sub(1, Ordering::Relaxed);
            return Err(StoreError::Exhausted);
        }
        if entry.email_verified_at_ms == 0 {
            return Err(StoreError::EmailUnverified);
        }
        entry.receives += 1;
        let remaining = entry.max_receives.saturating_sub(entry.receives);
        let payload_offset = entry.payload_offset;
        let payload_len = entry.payload_len;
        let expires_at_ms = entry.expires_at_ms;
        let receives = entry.receives;
        if remaining == 0 {
            index.remove(&code_hash);
            self.live.fetch_sub(1, Ordering::Relaxed);
        } else if cached {
            index.insert(code_hash, entry.clone());
        }
        let mut file = lock_store(&shard.file, "shard file")?;
        if remaining == 0 {
            append_tombstone(&mut file, &code_hash)?;
            self.append_bucket_tombstone(&code_hash)?;
            self.enqueue_replication(ReplicationEventKind::Tombstone {
                publish_code: publish_code.to_string(),
            });
        } else {
            append_receive_count(&mut file, &code_hash, receives)?;
            self.append_bucket_receive_count(&code_hash, receives)?;
            self.enqueue_replication(ReplicationEventKind::ReceiveCount {
                publish_code: publish_code.to_string(),
                receives,
            });
        }
        drop(index);
        let payload = read_payload(&mut file, payload_offset, payload_len)?;
        let email_verification = self.email_verification_for_receive(&entry);
        if remaining == 0 {
            lock_store(&self.verifications, "email verifications")?.remove(publish_code);
        }
        self.received.fetch_add(1, Ordering::Relaxed);
        Ok(ReceivedPublish {
            payload,
            expires_at_ms,
            remaining_receives: remaining,
            email_verification,
        })
    }

    pub fn receive_by_lookup(&self, lookup: &str) -> Result<ReceivedPublish, StoreError> {
        let publish_code = self.resolve_publish_lookup(lookup)?;
        self.receive(&publish_code)
    }

    pub fn verify_email(&self, publish_code: &str, token: &str) -> VerificationPage {
        match self.verify_email_inner(publish_code, token) {
            Ok(email) => VerificationPage {
                success: true,
                title: "Email verified".to_string(),
                message: "This email address is now attached to the pending reVault key publish. The contact still needs a second independent verification channel before fully trusting the key.".to_string(),
                email: Some(email),
            },
            Err(message) => VerificationPage {
                success: false,
                title: "Verification failed".to_string(),
                message,
                email: None,
            },
        }
    }

    pub fn delete(&self, publish_code: &str, delete_token: &[u8]) -> Result<bool, StoreError> {
        if !self.can_serve_publish_code(publish_code) {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return Ok(false);
        }
        let code_hash = self.code_hash(publish_code);
        let token_hash = self.delete_token_hash(delete_token);
        let shard = &self.shards[self.shard_for(&code_hash)];
        let mut index = lock_store(&shard.index, "shard index")?;
        let entry = match index.get(&code_hash).cloned() {
            Some(entry) => entry,
            None => match self.lookup_bucket(&code_hash)? {
                Some(entry) => entry,
                None => {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    return Ok(false);
                }
            },
        };
        if entry.delete_token_hash != token_hash {
            return Err(StoreError::DeleteTokenInvalid);
        }
        index.remove(&code_hash);
        let mut file = lock_store(&shard.file, "shard file")?;
        append_tombstone(&mut file, &code_hash)?;
        self.append_bucket_tombstone(&code_hash)?;
        lock_store(&self.verifications, "email verifications")?.remove(publish_code);
        self.deleted.fetch_add(1, Ordering::Relaxed);
        self.live.fetch_sub(1, Ordering::Relaxed);
        self.enqueue_replication(ReplicationEventKind::Tombstone {
            publish_code: publish_code.to_string(),
        });
        Ok(true)
    }

    pub fn delete_by_lookup(&self, lookup: &str, delete_token: &[u8]) -> Result<bool, StoreError> {
        self.delete(lookup, delete_token)
    }

    pub fn apply_replication_payload(&self, payload: &[u8]) -> Result<bool, StoreError> {
        let request = lockbox_publish_protocol::decode_replication_request(payload)
            .map_err(|err| StoreError::PayloadInvalid(err.to_string()))?;
        self.authorize_replication(&request)?;
        self.apply_replication_event(request.event)
    }

    pub fn apply_replication_event(&self, event: ReplicationEvent) -> Result<bool, StoreError> {
        if !self.reserve_replication_event(
            event.origin_server_id,
            event.origin_epoch,
            event.origin_sequence,
        )? {
            return Ok(false);
        }
        match event.kind {
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
            } => self.apply_replica_put(ReplicaPut {
                publish_code: &publish_code,
                delete_token_hash: &delete_token_hash,
                payload: &payload,
                contact_email: contact_email.as_deref(),
                expires_at_ms: expires_at_unix_ms,
                receive_ttl_ms,
                email_verified_at_ms: email_verified_at_unix_ms,
                max_receives,
                receives,
            })?,
            ReplicationEventKind::ReceiveCount {
                publish_code,
                receives,
            } => self.apply_replica_receive_count(&publish_code, receives)?,
            ReplicationEventKind::Tombstone { publish_code } => {
                self.apply_replica_tombstone(&publish_code)?
            }
            ReplicationEventKind::RateLimitBlock {
                client_ip,
                expires_at_unix_ms,
            } => {
                let peer_ip = client_ip.parse().map_err(|_| {
                    StoreError::PayloadInvalid(format!(
                        "invalid replicated rate limit client ip: {client_ip}"
                    ))
                })?;
                self.apply_rate_limit_block(peer_ip, expires_at_unix_ms)?;
            }
        }
        Ok(true)
    }

    fn apply_rate_limit_block(
        &self,
        peer_ip: IpAddr,
        expires_at_unix_ms: u64,
    ) -> Result<bool, StoreError> {
        let now_ms = unix_ms(SystemTime::now());
        if expires_at_unix_ms <= now_ms {
            return Ok(false);
        }
        let mut blocks = lock_store(&self.rate_limit_blocks, "rate limit blocks")?;
        let current = blocks.get(&peer_ip).copied().unwrap_or(0);
        if current >= expires_at_unix_ms {
            return Ok(false);
        }
        blocks.insert(peer_ip, expires_at_unix_ms);
        Ok(true)
    }

    fn authorize_replication(&self, request: &ReplicationRequest) -> Result<(), StoreError> {
        let Some(token) = &self.config.replication_token else {
            return Err(StoreError::ReplicationUnauthorized);
        };
        let expected = sign_replication_event(token.as_bytes(), &request.event);
        if request.authentication == expected {
            Ok(())
        } else {
            Err(StoreError::ReplicationUnauthorized)
        }
    }

    fn reserve_replication_event(
        &self,
        origin: u8,
        epoch: u64,
        sequence: u64,
    ) -> Result<bool, StoreError> {
        let mut state = lock_store(&self.replication_state, "replication state")?;
        let should_persist_for_gap = {
            let origin_state = state.origins.entry(origin).or_default();
            if epoch < origin_state.epoch {
                return Ok(false);
            }
            if epoch > origin_state.epoch {
                origin_state.epoch = epoch;
                origin_state.contiguous_sequence = 0;
                origin_state.gaps.clear();
            }
            if sequence <= origin_state.contiguous_sequence || origin_state.gaps.contains(&sequence)
            {
                return Ok(false);
            }

            let had_gaps = !origin_state.gaps.is_empty();
            if sequence == origin_state.contiguous_sequence.saturating_add(1) {
                origin_state.contiguous_sequence = sequence;
                while origin_state
                    .gaps
                    .remove(&origin_state.contiguous_sequence.saturating_add(1))
                {
                    origin_state.contiguous_sequence =
                        origin_state.contiguous_sequence.saturating_add(1);
                }
            } else {
                origin_state.gaps.insert(sequence);
            }
            had_gaps || !origin_state.gaps.is_empty()
        };

        state.accepted_since_persist = state.accepted_since_persist.saturating_add(1);
        if should_persist_for_gap
            || state.accepted_since_persist >= REPLICATION_STATE_PERSIST_INTERVAL
        {
            store_replication_state(&self.replication_state_path, &state)?;
            state.accepted_since_persist = 0;
        }
        Ok(true)
    }

    fn apply_replica_put(&self, put: ReplicaPut<'_>) -> Result<(), StoreError> {
        if put.delete_token_hash.len() != HASH_LEN {
            return Err(StoreError::PayloadInvalid(
                "delete token hash has invalid length".to_string(),
            ));
        }
        payload::validate_payload(put.payload)
            .map_err(|err| StoreError::PayloadInvalid(err.to_string()))?;
        let mut token_hash = [0_u8; HASH_LEN];
        token_hash.copy_from_slice(put.delete_token_hash);
        let contact_email = match put.contact_email {
            Some(email) => Some(normalize_verification_email(email)?),
            None => None,
        };
        let code_hash = self.code_hash(put.publish_code);
        let shard = &self.shards[self.shard_for(&code_hash)];
        let mut entry = PublishEntry {
            publish_code: put.publish_code.to_string(),
            delete_token_hash: token_hash,
            contact_email,
            payload_offset: 0,
            payload_len: put.payload.len() as u32,
            expires_at_ms: put.expires_at_ms,
            receive_ttl_ms: put.receive_ttl_ms,
            email_verified_at_ms: put.email_verified_at_ms,
            max_receives: put.max_receives,
            receives: put.receives,
        };
        let mut index = lock_store(&shard.index, "shard index")?;
        let existed = index.contains_key(&code_hash) || self.lookup_bucket(&code_hash)?.is_some();
        let mut file = lock_store(&shard.file, "shard file")?;
        let (payload_offset, payload_len) = append_put(&mut file, &code_hash, &entry, put.payload)?;
        entry.payload_offset = payload_offset;
        entry.payload_len = payload_len;
        self.append_bucket_put(&code_hash, &entry)?;
        if index.len() < self.config.index_cache_entries / self.shards.len().max(1) {
            index.insert(code_hash, entry);
        }
        if !existed {
            self.live.fetch_add(1, Ordering::Relaxed);
        }
        lock_store(&shard.expiry_buckets, "expiry buckets")?.push_back((
            put.expires_at_ms,
            vec![(code_hash, put.publish_code.to_string())],
        ));
        Ok(())
    }

    fn apply_replica_receive_count(
        &self,
        publish_code: &str,
        receives: u16,
    ) -> Result<(), StoreError> {
        let code_hash = self.code_hash(publish_code);
        let shard = &self.shards[self.shard_for(&code_hash)];
        if self.lookup_bucket(&code_hash)?.is_some() {
            if let Some(cached) = lock_store(&shard.index, "shard index")?.get_mut(&code_hash) {
                cached.receives = receives;
            }
        }
        let mut file = lock_store(&shard.file, "shard file")?;
        append_receive_count(&mut file, &code_hash, receives)?;
        self.append_bucket_receive_count(&code_hash, receives)?;
        Ok(())
    }

    fn apply_replica_tombstone(&self, publish_code: &str) -> Result<(), StoreError> {
        let code_hash = self.code_hash(publish_code);
        let shard = &self.shards[self.shard_for(&code_hash)];
        let removed = lock_store(&shard.index, "shard index")?
            .remove(&code_hash)
            .is_some()
            || self.lookup_bucket(&code_hash)?.is_some();
        let mut file = lock_store(&shard.file, "shard file")?;
        append_tombstone(&mut file, &code_hash)?;
        self.append_bucket_tombstone(&code_hash)?;
        if removed {
            self.live.fetch_sub(1, Ordering::Relaxed);
        }
        Ok(())
    }

    fn can_serve_publish_code(&self, publish_code: &str) -> bool {
        let Some((owner_id, secondary_id)) = publish_code_locator(publish_code) else {
            return false;
        };
        owner_id == self.config.server_id
            || secondary_id == self.config.server_id
            || self.config.promoted_owner_ids.contains(&owner_id)
    }

    fn enqueue_replication(&self, kind: ReplicationEventKind) {
        if let Some(tx) = &self.replication_tx {
            let _ = tx.send(kind);
        }
    }

    pub fn purge_expired(&self) -> usize {
        let now_ms = unix_ms(SystemTime::now());
        let mut purged = 0;
        for shard in &self.shards {
            let mut due = Vec::new();
            {
                let Ok(mut buckets) = shard.expiry_buckets.lock() else {
                    continue;
                };
                while let Some((expires_at, _)) = buckets.front() {
                    if *expires_at > now_ms {
                        break;
                    }
                    if let Some((_, entries)) = buckets.pop_front() {
                        due.extend(entries);
                    }
                }
            }
            if due.is_empty() {
                continue;
            }
            let Ok(mut index) = shard.index.lock() else {
                continue;
            };
            let Ok(mut file) = shard.file.lock() else {
                continue;
            };
            for (hash, publish_code) in due {
                let should_remove = index
                    .get(&hash)
                    .map(|entry| entry.expires_at_ms <= now_ms)
                    .unwrap_or(false);
                if should_remove {
                    index.remove(&hash);
                    let _ = append_tombstone(&mut file, &hash);
                    let _ = self.append_bucket_tombstone(&hash);
                    self.enqueue_replication(ReplicationEventKind::Tombstone { publish_code });
                    purged += 1;
                }
            }
        }
        if purged > 0 {
            self.expired.fetch_add(purged as u64, Ordering::Relaxed);
            self.live.fetch_sub(purged, Ordering::Relaxed);
        }
        let now_ms = unix_ms(SystemTime::now());
        if let Ok(mut verifications) = self.verifications.lock() {
            verifications.retain(|_, entry| entry.expires_at_ms > now_ms);
        }
        if let Ok(mut blocks) = self.rate_limit_blocks.lock() {
            blocks.retain(|_, expires_at| *expires_at > now_ms);
        }
        purged
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            created: self.created.load(Ordering::Relaxed),
            received: self.received.load(Ordering::Relaxed),
            deleted: self.deleted.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            live: self.live.load(Ordering::Relaxed),
            segment_bytes: self.segment_bytes(),
            replication_pending: load_outbox_pending(&self.replication_outbox_path)
                .map(|pending| pending.len())
                .unwrap_or(0),
            replication_last_sequence: load_replication_sequence(&self.replication_sequence_path)
                .unwrap_or(0),
        }
    }

    pub fn status_document(&self) -> lockbox_publish_protocol::KeyServerStatus {
        let now = Instant::now();
        if let Ok(cache) = self.status_cache.lock() {
            if let Some(cache) = cache.as_ref() {
                if now.duration_since(cache.cached_at) < STATUS_CACHE_TTL {
                    return cache.document.clone();
                }
            }
        }
        let stats = self.stats();
        let document = lockbox_publish_protocol::KeyServerStatus {
            created: stats.created,
            received: stats.received,
            deleted: stats.deleted,
            expired: stats.expired,
            misses: stats.misses,
            live: stats.live as u64,
            segment_bytes: stats.segment_bytes,
            replication_pending: stats.replication_pending as u64,
            replication_last_sequence: stats.replication_last_sequence,
        };
        if let Ok(mut cache) = self.status_cache.lock() {
            *cache = Some(StatusCache {
                cached_at: now,
                document: document.clone(),
            });
        }
        document
    }

    pub fn resync_peer(&self, peer_url: &str) -> Result<usize, StoreError> {
        let token = self.config.replication_token.as_ref().ok_or_else(|| {
            StoreError::Config("replication_token is required for resync".to_string())
        })?;
        let mut sequence = load_replication_sequence(&self.replication_sequence_path)?;
        let mut sent = 0usize;
        for shard in &self.shards {
            let snapshot = lock_store(&shard.index, "shard index")?
                .iter()
                .map(|(hash, entry)| (*hash, entry.clone()))
                .collect::<Vec<_>>();
            if snapshot.is_empty() {
                continue;
            }
            let mut file = lock_store(&shard.file, "shard file")?;
            for (_, entry) in snapshot {
                if entry.expires_at_ms <= unix_ms(SystemTime::now()) {
                    continue;
                }
                let payload = read_payload(&mut file, entry.payload_offset, entry.payload_len)?;
                sequence = sequence.saturating_add(1);
                store_replication_sequence(&self.replication_sequence_path, sequence)?;
                let event = ReplicationEvent {
                    origin_server_id: self.config.server_id,
                    origin_epoch: self.config.origin_epoch,
                    origin_sequence: sequence,
                    kind: ReplicationEventKind::PutPublish {
                        publish_code: entry.publish_code.clone(),
                        delete_token_hash: entry.delete_token_hash.to_vec(),
                        payload,
                        contact_email: entry.contact_email.clone(),
                        expires_at_unix_ms: entry.expires_at_ms,
                        receive_ttl_ms: entry.receive_ttl_ms,
                        email_verified_at_unix_ms: entry.email_verified_at_ms,
                        max_receives: entry.max_receives,
                        receives: entry.receives,
                    },
                };
                let request = encode_replication_request(&ReplicationRequest {
                    authentication: sign_replication_event(token.as_bytes(), &event),
                    event,
                });
                append_outbox_event(&self.replication_outbox_path, sequence, &request)?;
                if let Err(err) = send_replication_request(
                    &[peer_url.to_string()],
                    &request,
                    self.config.topology_token.as_deref(),
                ) {
                    return Err(StoreError::Io(std::io::Error::other(format!(
                        "replication peer {peer_url} did not accept resync event {sequence}: {err}"
                    ))));
                } else {
                    append_outbox_ack(&self.replication_outbox_path, sequence)?;
                    sent += 1;
                }
            }
        }
        Ok(sent)
    }

    pub fn segment_bytes(&self) -> u64 {
        self.shards
            .iter()
            .filter_map(|shard| shard.path.metadata().ok())
            .map(|metadata| metadata.len())
            .sum()
    }

    pub fn compact_if_needed(&self) -> Result<CompactionReport, StoreError> {
        if self.live.load(Ordering::Relaxed) > self.config.index_cache_entries {
            return Ok(CompactionReport::default());
        }
        let mut report = CompactionReport::default();
        for shard in &self.shards {
            let segment_bytes = shard.path.metadata().map(|m| m.len()).unwrap_or(0);
            if segment_bytes < self.config.compact_min_bytes {
                continue;
            }
            let live_bytes = {
                let index = lock_store(&shard.index, "shard index")?;
                compacted_bytes_for_index(&index)
            };
            if live_bytes == 0 || live_bytes.saturating_mul(2) < segment_bytes {
                report.add(compact_shard(shard)?);
            }
        }
        Ok(report)
    }

    pub fn compact(&self) -> Result<CompactionReport, StoreError> {
        if self.live.load(Ordering::Relaxed) > self.config.index_cache_entries {
            return Ok(CompactionReport::default());
        }
        let mut report = CompactionReport::default();
        for shard in &self.shards {
            report.add(compact_shard(shard)?);
        }
        Ok(report)
    }

    fn publish_code_locator(&self) -> (u8, u8) {
        let topology = self.topology();
        let primary_id = topology
            .routes
            .iter()
            .find(|route| route.owner_id == self.config.server_id)
            .map(|route| route.primary_id)
            .unwrap_or(self.config.server_id);
        let secondary_id = topology
            .routes
            .iter()
            .find(|route| route.owner_id == self.config.server_id)
            .and_then(|route| route.failover_ids.first().copied())
            .or_else(|| {
                topology
                    .servers
                    .iter()
                    .filter(|server| server.status != ServerStatus::Disabled)
                    .find(|server| server.id != self.config.server_id)
                    .map(|server| server.id)
            })
            .unwrap_or(primary_id);
        (primary_id, secondary_id)
    }

    fn generate_unique_code(&self) -> Result<String, StoreError> {
        let space = 10_u64.pow(SHARE_CODE_BODY_DIGITS as u32);
        let (primary_id, secondary_id) = self.publish_code_locator();
        for _ in 0..100 {
            let mut random = [0_u8; 8];
            getrandom(&mut random)
                .map_err(|err| StoreError::Io(std::io::Error::other(err.to_string())))?;
            let value = u64::from_be_bytes(random) % space;
            if let Some(code) = self.unique_code_from_value(primary_id, secondary_id, value)? {
                return Ok(code);
            }
        }
        Err(StoreError::Io(std::io::Error::other(
            "unable to allocate unique publish code",
        )))
    }

    fn unique_code_from_value(
        &self,
        primary_id: u8,
        secondary_id: u8,
        value: u64,
    ) -> Result<Option<String>, StoreError> {
        let primary = publish_code_server_id_char(primary_id)
            .ok_or_else(|| StoreError::Config("invalid primary server id".to_string()))?;
        let secondary = publish_code_server_id_char(secondary_id)
            .ok_or_else(|| StoreError::Config("invalid secondary server id".to_string()))?;
        let code = format!(
            "{}{}{:0width$}",
            primary as char,
            secondary as char,
            value % 10_u64.pow(SHARE_CODE_BODY_DIGITS as u32),
            width = SHARE_CODE_BODY_DIGITS
        );
        let hash = self.code_hash(&code);
        let shard = &self.shards[self.shard_for(&hash)];
        if lock_store(&shard.index, "shard index")?.contains_key(&hash) {
            return Ok(None);
        }
        if self.lookup_bucket(&hash)?.is_some() {
            return Ok(None);
        }
        Ok(Some(code))
    }

    fn code_hash(&self, code: &str) -> RecordHash {
        keyed_hash(&self.secret, b"publish-code", code.as_bytes())
    }

    fn delete_token_hash(&self, token: &[u8]) -> RecordHash {
        stable_hash(b"delete-token", token)
    }

    fn shard_for(&self, code_hash: &RecordHash) -> usize {
        let raw = u32::from_be_bytes([code_hash[0], code_hash[1], code_hash[2], code_hash[3]]);
        raw as usize % self.shards.len()
    }

    fn bucket_path(&self, code_hash: &RecordHash) -> PathBuf {
        self.bucket_dir
            .join(format!("bucket-{:03x}.idx", bucket_for_hash(code_hash)))
    }

    fn append_bucket_put(
        &self,
        code_hash: &RecordHash,
        entry: &PublishEntry,
    ) -> Result<(), StoreError> {
        let mut record = [0_u8; BUCKET_RECORD_LEN];
        record[0] = BUCKET_PUT;
        record[1..1 + HASH_LEN].copy_from_slice(code_hash);
        record[17..17 + HASH_LEN].copy_from_slice(&entry.delete_token_hash);
        record[33..41].copy_from_slice(&entry.payload_offset.to_be_bytes());
        record[41..45].copy_from_slice(&entry.payload_len.to_be_bytes());
        record[45..53].copy_from_slice(&entry.expires_at_ms.to_be_bytes());
        record[53..55].copy_from_slice(&entry.max_receives.to_be_bytes());
        record[55..57].copy_from_slice(&entry.receives.to_be_bytes());
        record[57..65].copy_from_slice(&entry.receive_ttl_ms.to_be_bytes());
        record[65..73].copy_from_slice(&entry.email_verified_at_ms.to_be_bytes());
        self.append_bucket_record(code_hash, &record)
    }

    fn append_bucket_tombstone(&self, code_hash: &RecordHash) -> Result<(), StoreError> {
        let mut record = [0_u8; BUCKET_RECORD_LEN];
        record[0] = BUCKET_TOMBSTONE;
        record[1..1 + HASH_LEN].copy_from_slice(code_hash);
        self.append_bucket_record(code_hash, &record)
    }

    fn append_bucket_receive_count(
        &self,
        code_hash: &RecordHash,
        receives: u16,
    ) -> Result<(), StoreError> {
        let mut record = [0_u8; BUCKET_RECORD_LEN];
        record[0] = BUCKET_RECEIVE_COUNT;
        record[1..1 + HASH_LEN].copy_from_slice(code_hash);
        record[55..57].copy_from_slice(&receives.to_be_bytes());
        self.append_bucket_record(code_hash, &record)
    }

    fn append_bucket_record(
        &self,
        code_hash: &RecordHash,
        record: &[u8; BUCKET_RECORD_LEN],
    ) -> Result<(), StoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.bucket_path(code_hash))?;
        file.write_all(record)?;
        Ok(())
    }

    fn lookup_bucket(&self, code_hash: &RecordHash) -> Result<Option<PublishEntry>, StoreError> {
        let path = self.bucket_path(code_hash);
        if !path.exists() {
            return Ok(None);
        }
        let mut file = OpenOptions::new().read(true).open(path)?;
        let records = file.metadata()?.len() as usize / BUCKET_RECORD_LEN;
        let mut latest_receives = None;
        let mut record = [0_u8; BUCKET_RECORD_LEN];
        for index in (0..records).rev() {
            file.seek(SeekFrom::Start((index * BUCKET_RECORD_LEN) as u64))?;
            file.read_exact(&mut record)?;
            if record[1..1 + HASH_LEN] != code_hash[..] {
                continue;
            }
            match record[0] {
                BUCKET_TOMBSTONE => return Ok(None),
                BUCKET_RECEIVE_COUNT => {
                    latest_receives = Some(u16::from_be_bytes([record[55], record[56]]));
                }
                BUCKET_PUT => {
                    let mut delete_token_hash = [0_u8; HASH_LEN];
                    delete_token_hash.copy_from_slice(&record[17..17 + HASH_LEN]);
                    let receives =
                        latest_receives.unwrap_or(u16::from_be_bytes([record[55], record[56]]));
                    return Ok(Some(PublishEntry {
                        publish_code: String::new(),
                        delete_token_hash,
                        contact_email: None,
                        payload_offset: u64::from_be_bytes([
                            record[33], record[34], record[35], record[36], record[37], record[38],
                            record[39], record[40],
                        ]),
                        payload_len: u32::from_be_bytes([
                            record[41], record[42], record[43], record[44],
                        ]),
                        expires_at_ms: u64::from_be_bytes([
                            record[45], record[46], record[47], record[48], record[49], record[50],
                            record[51], record[52],
                        ]),
                        receive_ttl_ms: u64::from_be_bytes([
                            record[57], record[58], record[59], record[60], record[61], record[62],
                            record[63], record[64],
                        ]),
                        email_verified_at_ms: u64::from_be_bytes([
                            record[65], record[66], record[67], record[68], record[69], record[70],
                            record[71], record[72],
                        ]),
                        max_receives: u16::from_be_bytes([record[53], record[54]]),
                        receives,
                    }));
                }
                _ => {}
            }
        }
        Ok(None)
    }
}

fn bucket_for_hash(code_hash: &RecordHash) -> usize {
    let raw = u16::from_be_bytes([code_hash[0], code_hash[1]]) as usize;
    raw % BUCKET_COUNT
}

#[derive(Debug)]
pub struct StoreStats {
    pub created: u64,
    pub received: u64,
    pub deleted: u64,
    pub expired: u64,
    pub misses: u64,
    pub live: usize,
    pub segment_bytes: u64,
    pub replication_pending: usize,
    pub replication_last_sequence: u64,
}

#[derive(Debug, Default)]
pub struct CompactionReport {
    pub shards_compacted: usize,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub live_records: usize,
}

impl CompactionReport {
    fn add(&mut self, other: Self) {
        self.shards_compacted += other.shards_compacted;
        self.bytes_before += other.bytes_before;
        self.bytes_after += other.bytes_after;
        self.live_records += other.live_records;
    }
}

fn prune_rate_bucket(bucket: &mut VecDeque<u64>, cutoff_ms: u64) {
    while matches!(bucket.front(), Some(value) if *value < cutoff_ms) {
        bucket.pop_front();
    }
}

fn load_or_create_secret(state_dir: &std::path::Path) -> Result<[u8; 32], StoreError> {
    let path = state_dir.join("server.secret");
    if path.exists() {
        restrict_secret_file_permissions(&path)?;
        return load_existing_secret(&path);
    }
    let mut secret = [0_u8; 32];
    getrandom(&mut secret).map_err(|err| StoreError::Io(std::io::Error::other(err.to_string())))?;
    match write_secret_file(&path, &secret) {
        Ok(()) => Ok(secret),
        Err(StoreError::Io(err)) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            restrict_secret_file_permissions(&path)?;
            load_existing_secret(&path)
        }
        Err(err) => Err(err),
    }
}

fn load_existing_secret(path: &std::path::Path) -> Result<[u8; 32], StoreError> {
    let mut bytes = fs::read(path)?;
    if bytes.len() < 32 {
        return Err(StoreError::Io(std::io::Error::other(
            "server secret is too short",
        )));
    }
    bytes.truncate(32);
    let mut secret = [0_u8; 32];
    secret.copy_from_slice(&bytes);
    Ok(secret)
}

fn write_secret_file(path: &std::path::Path, secret: &[u8; 32]) -> Result<(), StoreError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(secret)?;
    file.sync_data()?;
    Ok(())
}

fn restrict_secret_file_permissions(path: &std::path::Path) -> Result<(), StoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn load_replication_state(path: &Path) -> Result<ReplicationState, StoreError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ReplicationState::default());
        }
        Err(err) => return Err(StoreError::Io(err)),
    };
    if !bytes.starts_with(REPLICATION_STATE_MAGIC) {
        return Ok(ReplicationState::default());
    }
    let mut offset = REPLICATION_STATE_MAGIC.len();
    let origin_count = read_u32(&bytes, &mut offset)? as usize;
    let mut state = ReplicationState::default();
    for _ in 0..origin_count {
        let origin = read_u8(&bytes, &mut offset)?;
        let epoch = read_u64(&bytes, &mut offset)?;
        let contiguous_sequence = read_u64(&bytes, &mut offset)?;
        let gap_count = read_u32(&bytes, &mut offset)? as usize;
        let mut gaps = HashSet::with_capacity(gap_count);
        for _ in 0..gap_count {
            gaps.insert(read_u64(&bytes, &mut offset)?);
        }
        state.origins.insert(
            origin,
            ReplicationOriginState {
                epoch,
                contiguous_sequence,
                gaps,
            },
        );
    }
    Ok(state)
}

fn store_replication_state(path: &Path, state: &ReplicationState) -> Result<(), StoreError> {
    let mut bytes = Vec::with_capacity(16 + state.origins.len() * 32);
    bytes.extend_from_slice(REPLICATION_STATE_MAGIC);
    bytes.extend_from_slice(&(state.origins.len() as u32).to_be_bytes());
    for (origin, origin_state) in &state.origins {
        bytes.push(*origin);
        bytes.extend_from_slice(&origin_state.epoch.to_be_bytes());
        bytes.extend_from_slice(&origin_state.contiguous_sequence.to_be_bytes());
        bytes.extend_from_slice(&(origin_state.gaps.len() as u32).to_be_bytes());
        for gap in &origin_state.gaps {
            bytes.extend_from_slice(&gap.to_be_bytes());
        }
    }
    let mut tmp_path = path.to_path_buf();
    tmp_path.set_extension("bin.tmp");
    fs::write(&tmp_path, bytes)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn read_u8(bytes: &[u8], offset: &mut usize) -> Result<u8, StoreError> {
    if *offset >= bytes.len() {
        return Err(StoreError::Io(std::io::Error::other(
            "truncated replication state",
        )));
    }
    let value = bytes[*offset];
    *offset += 1;
    Ok(value)
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, StoreError> {
    let end = offset.saturating_add(4);
    if end > bytes.len() {
        return Err(StoreError::Io(std::io::Error::other(
            "truncated replication state",
        )));
    }
    let value = u32::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset = end;
    Ok(value)
}

fn read_u64(bytes: &[u8], offset: &mut usize) -> Result<u64, StoreError> {
    let end = offset.saturating_add(8);
    if end > bytes.len() {
        return Err(StoreError::Io(std::io::Error::other(
            "truncated replication state",
        )));
    }
    let value = u64::from_be_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
        bytes[*offset + 4],
        bytes[*offset + 5],
        bytes[*offset + 6],
        bytes[*offset + 7],
    ]);
    *offset = end;
    Ok(value)
}

fn start_replication_worker(
    config: &ServerConfig,
    outbox_path: &Path,
    sequence_path: &Path,
) -> Option<mpsc::SyncSender<ReplicationEventKind>> {
    let token = config.replication_token.clone()?;
    if config.replication_peer_urls.is_empty() {
        return None;
    }
    let peer_urls = config.replication_peer_urls.clone();
    let server_token = config.topology_token.clone();
    let origin_server_id = config.server_id;
    let origin_epoch = config.origin_epoch;
    let outbox_path = outbox_path.to_path_buf();
    let sequence_path = sequence_path.to_path_buf();
    let (tx, rx) = mpsc::sync_channel::<ReplicationEventKind>(8192);
    thread::Builder::new()
        .name("publish-replication".to_string())
        .stack_size(256 * 1024)
        .spawn(move || {
            let mut sequence = load_replication_sequence(&sequence_path).unwrap_or(0);
            let mut pending = load_outbox_pending(&outbox_path).unwrap_or_else(|err| {
                log_server_event(format!("replication outbox load failed: {err}"));
                VecDeque::new()
            });
            let mut last_retry_log = Instant::now() - Duration::from_secs(30);
            loop {
                let timeout = if pending.is_empty() {
                    Duration::from_secs(1)
                } else {
                    Duration::from_millis(10)
                };
                match rx.recv_timeout(timeout) {
                    Ok(kind) => queue_replication_event(
                        kind,
                        ReplicationQueue {
                            sequence: &mut sequence,
                            sequence_path: &sequence_path,
                            outbox_path: &outbox_path,
                            pending: &mut pending,
                            origin_server_id,
                            origin_epoch,
                            token: token.as_bytes(),
                        },
                    ),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                for kind in rx.try_iter().take(8192) {
                    queue_replication_event(
                        kind,
                        ReplicationQueue {
                            sequence: &mut sequence,
                            sequence_path: &sequence_path,
                            outbox_path: &outbox_path,
                            pending: &mut pending,
                            origin_server_id,
                            origin_epoch,
                            token: token.as_bytes(),
                        },
                    );
                }
                retry_pending_outbox(
                    &outbox_path,
                    &peer_urls,
                    server_token.as_deref(),
                    &mut pending,
                    &mut last_retry_log,
                );
            }
        })
        .ok()?;
    Some(tx)
}

fn queue_replication_event(kind: ReplicationEventKind, queue: ReplicationQueue<'_>) {
    *queue.sequence = queue.sequence.saturating_add(1);
    if let Err(err) = store_replication_sequence(queue.sequence_path, *queue.sequence) {
        log_server_event(format!("replication sequence persist failed: {err}"));
        return;
    }
    let event = ReplicationEvent {
        origin_server_id: queue.origin_server_id,
        origin_epoch: queue.origin_epoch,
        origin_sequence: *queue.sequence,
        kind,
    };
    let request = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(queue.token, &event),
        event,
    });
    if let Err(err) = append_outbox_event(queue.outbox_path, *queue.sequence, &request) {
        log_server_event(format!("replication outbox append failed: {err}"));
        return;
    }
    queue.pending.push_back((*queue.sequence, request));
}

fn retry_pending_outbox(
    outbox_path: &Path,
    peer_urls: &[String],
    server_token: Option<&str>,
    pending: &mut VecDeque<(u64, Vec<u8>)>,
    last_retry_log: &mut Instant,
) {
    let attempted = pending.len();
    let mut failed = 0usize;
    let mut first_failure = None;
    let mut remaining = VecDeque::new();
    while let Some((sequence, request)) = pending.pop_front() {
        match send_replication_request(peer_urls, &request, server_token) {
            Ok(()) => {
                if let Err(err) = append_outbox_ack(outbox_path, sequence) {
                    log_server_event(format!("replication outbox ack failed: {err}"));
                    remaining.push_back((sequence, request));
                }
            }
            Err(err) => {
                failed += 1;
                if first_failure.is_none() {
                    first_failure = Some(err);
                }
                remaining.push_back((sequence, request));
                remaining.append(pending);
                break;
            }
        }
    }
    if failed > 0 && last_retry_log.elapsed() >= Duration::from_secs(10) {
        let first_failure = first_failure.unwrap_or_else(|| "unknown failure".to_string());
        log_server_event(format!(
            "replication retry deferred {failed}/{attempted} pending event(s) for {} peer(s); first failure: {first_failure}",
            peer_urls.len()
        ));
        *last_retry_log = Instant::now();
    }
    *pending = remaining;
}

fn send_replication_request(
    peer_urls: &[String],
    request: &[u8],
    server_token: Option<&str>,
) -> Result<(), String> {
    let mut first_failure = None;
    for peer_url in peer_urls {
        match HttpTransport::new(peer_url).and_then(|transport| {
            let response = if let Some(server_token) = server_token {
                transport.post_binary_with_server_token(request, server_token)?
            } else {
                transport.post_binary(request)?
            };
            protocol::decode_response(&response, 1024)
                .map_err(lockbox_publish_protocol::ClientError::from)
        }) {
            Ok(response) if response.status == Status::Success => {}
            Ok(response) => {
                if first_failure.is_none() {
                    first_failure = Some(format!(
                        "replication peer {peer_url} returned {:?}",
                        response.status
                    ));
                }
            }
            Err(err) => {
                if first_failure.is_none() {
                    first_failure = Some(format!("replication peer {peer_url} failed: {err}"));
                }
            }
        }
    }
    first_failure.map_or(Ok(()), Err)
}

fn load_replication_sequence(path: &Path) -> Result<u64, StoreError> {
    match fs::read(path) {
        Ok(bytes) if bytes.len() >= 8 => Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])),
        Ok(_) => Ok(0),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(StoreError::Io(err)),
    }
}

fn store_replication_sequence(path: &Path, sequence: u64) -> Result<(), StoreError> {
    fs::write(path, sequence.to_be_bytes()).map_err(StoreError::Io)
}

fn append_outbox_event(path: &Path, sequence: u64, request: &[u8]) -> Result<(), StoreError> {
    let mut body = Vec::with_capacity(8 + 4 + request.len());
    body.extend_from_slice(&sequence.to_be_bytes());
    protocol::put_bytes(&mut body, request);
    append_outbox_record(path, OUTBOX_EVENT, &body)
}

fn append_outbox_ack(path: &Path, sequence: u64) -> Result<(), StoreError> {
    append_outbox_record(path, OUTBOX_ACK, &sequence.to_be_bytes())
}

fn append_outbox_record(path: &Path, kind: u16, body: &[u8]) -> Result<(), StoreError> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut header = Vec::with_capacity(OUTBOX_HEADER_LEN);
    header.extend_from_slice(OUTBOX_MAGIC);
    header.extend_from_slice(&1_u16.to_be_bytes());
    header.extend_from_slice(&kind.to_be_bytes());
    header.extend_from_slice(&(body.len() as u32).to_be_bytes());
    header.extend_from_slice(&checksum(body).to_be_bytes());
    file.write_all(&header)?;
    file.write_all(body)?;
    Ok(())
}

fn load_outbox_pending(path: &Path) -> Result<VecDeque<(u64, Vec<u8>)>, StoreError> {
    let mut file = match OpenOptions::new().read(true).open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(VecDeque::new()),
        Err(err) => return Err(StoreError::Io(err)),
    };
    let mut events = HashMap::<u64, Vec<u8>>::new();
    let mut acks = HashSet::<u64>::new();
    let mut header = [0_u8; OUTBOX_HEADER_LEN];
    loop {
        match file.read_exact(&mut header) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(StoreError::Io(err)),
        }
        if &header[0..4] != OUTBOX_MAGIC {
            break;
        }
        let kind = u16::from_be_bytes([header[6], header[7]]);
        let len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]) as usize;
        let expected = u32::from_be_bytes([header[12], header[13], header[14], header[15]]);
        let mut body = vec![0_u8; len];
        if file.read_exact(&mut body).is_err() {
            break;
        }
        if checksum(&body) != expected {
            break;
        }
        match kind {
            OUTBOX_EVENT if body.len() >= 12 => {
                let sequence = u64::from_be_bytes([
                    body[0], body[1], body[2], body[3], body[4], body[5], body[6], body[7],
                ]);
                let request_len =
                    u32::from_be_bytes([body[8], body[9], body[10], body[11]]) as usize;
                if body.len() == 12 + request_len {
                    events.insert(sequence, body[12..].to_vec());
                }
            }
            OUTBOX_ACK if body.len() == 8 => {
                acks.insert(u64::from_be_bytes([
                    body[0], body[1], body[2], body[3], body[4], body[5], body[6], body[7],
                ]));
            }
            _ => {}
        }
    }
    let mut pending = events
        .into_iter()
        .filter(|(sequence, _)| !acks.contains(sequence))
        .collect::<Vec<_>>();
    pending.sort_by_key(|(sequence, _)| *sequence);
    Ok(VecDeque::from(pending))
}

fn keyed_hash(secret: &[u8; 32], domain: &[u8], value: &[u8]) -> RecordHash {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    hasher.update(domain);
    hasher.update(value);
    let full_hash: [u8; 32] = hasher.finalize().into();
    let mut out = [0_u8; HASH_LEN];
    out.copy_from_slice(&full_hash[..HASH_LEN]);
    out
}

fn stable_hash(domain: &[u8], value: &[u8]) -> RecordHash {
    let mut hasher = Sha256::new();
    hasher.update(b"lockbox-key-server-stable-hash-v1");
    hasher.update(domain);
    hasher.update(value);
    let full_hash: [u8; 32] = hasher.finalize().into();
    let mut out = [0_u8; HASH_LEN];
    out.copy_from_slice(&full_hash[..HASH_LEN]);
    out
}

fn normalize_verification_email(email: &str) -> Result<String, StoreError> {
    payload::normalize_contact_email(email)
        .map_err(|_| StoreError::PayloadInvalid("verification email is invalid".to_string()))
}

fn render_email_template(
    template: &str,
    email: &str,
    publish_code: &str,
    verification_url: &str,
) -> String {
    template
        .replace("{email}", email)
        .replace("{publish_code}", publish_code)
        .replace("{verification_url}", verification_url)
}

fn verification_query_parts(url: &str) -> Option<(String, String)> {
    let query = url.split_once('?')?.1;
    let mut code = None;
    let mut token = None;
    for part in query.split('&') {
        let (key, value) = part.split_once('=')?;
        match key {
            "code" => code = Some(value.to_string()),
            "token" => token = Some(value.to_string()),
            _ => {}
        }
    }
    Some((code?, token?))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn append_put(
    file: &mut File,
    code_hash: &RecordHash,
    entry: &PublishEntry,
    payload: &[u8],
) -> Result<(u64, u32), StoreError> {
    if entry.publish_code.len() > u8::MAX as usize {
        return Err(StoreError::Config("publish code is too long".to_string()));
    }
    let prefix_len = put_record_payload_offset(&entry.publish_code);
    let mut body = Vec::with_capacity(prefix_len + payload.len());
    body.extend_from_slice(code_hash);
    body.push(entry.publish_code.len() as u8);
    body.extend_from_slice(entry.publish_code.as_bytes());
    body.extend_from_slice(&entry.delete_token_hash);
    body.extend_from_slice(&entry.expires_at_ms.to_be_bytes());
    body.extend_from_slice(&entry.receive_ttl_ms.to_be_bytes());
    body.extend_from_slice(&entry.email_verified_at_ms.to_be_bytes());
    body.extend_from_slice(&entry.max_receives.to_be_bytes());
    protocol::put_bytes(&mut body, payload);
    if let Some(email) = &entry.contact_email {
        protocol::put_string(&mut body, email);
    }
    let body_offset = append_record(file, KIND_PUT, &body)?;
    Ok((body_offset + prefix_len as u64, payload.len() as u32))
}

fn append_tombstone(file: &mut File, code_hash: &RecordHash) -> Result<(), StoreError> {
    append_record(file, KIND_TOMBSTONE, code_hash).map(|_| ())
}

fn append_receive_count(
    file: &mut File,
    code_hash: &RecordHash,
    receives: u16,
) -> Result<(), StoreError> {
    let mut body = Vec::with_capacity(HASH_LEN + 2);
    body.extend_from_slice(code_hash);
    body.extend_from_slice(&receives.to_be_bytes());
    append_record(file, KIND_RECEIVE_COUNT, &body).map(|_| ())
}

fn append_record(file: &mut File, kind: u16, body: &[u8]) -> Result<u64, StoreError> {
    let record_start = file.seek(SeekFrom::End(0))?;
    let mut header = Vec::with_capacity(RECORD_HEADER_LEN);
    header.extend_from_slice(RECORD_MAGIC);
    header.extend_from_slice(&1_u16.to_be_bytes());
    header.extend_from_slice(&kind.to_be_bytes());
    header.extend_from_slice(&(RECORD_HEADER_LEN as u16).to_be_bytes());
    header.extend_from_slice(&0_u16.to_be_bytes());
    header.extend_from_slice(&(body.len() as u32).to_be_bytes());
    header.extend_from_slice(&checksum(body).to_be_bytes());
    file.write_all(&header)?;
    file.write_all(body)?;
    Ok(record_start + RECORD_HEADER_LEN as u64)
}

fn replay(file: &mut File) -> Result<HashMap<RecordHash, PublishEntry>, StoreError> {
    file.seek(SeekFrom::Start(0))?;
    let mut index = HashMap::new();
    let mut header = [0_u8; RECORD_HEADER_LEN];
    let now_ms = unix_ms(SystemTime::now());
    loop {
        let record_start = file.stream_position()?;
        match file.read_exact(&mut header) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(StoreError::Io(err)),
        }
        if &header[0..4] != RECORD_MAGIC {
            break;
        }
        let kind = u16::from_be_bytes([header[6], header[7]]);
        let len = u32::from_be_bytes([header[12], header[13], header[14], header[15]]) as usize;
        let expected = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
        let mut body = vec![0_u8; len];
        if file.read_exact(&mut body).is_err() {
            break;
        }
        if checksum(&body) != expected {
            break;
        }
        match kind {
            KIND_PUT => {
                if body.len() < HASH_LEN + 1 {
                    continue;
                }
                let mut code_hash = [0_u8; HASH_LEN];
                code_hash.copy_from_slice(&body[0..HASH_LEN]);
                let code_len = body[HASH_LEN] as usize;
                let code_start = HASH_LEN + 1;
                let code_end = code_start + code_len;
                if body.len() < code_end + HASH_LEN + 8 + 8 + 8 + 2 + 4 {
                    continue;
                }
                let Ok(publish_code) = std::str::from_utf8(&body[code_start..code_end]) else {
                    continue;
                };
                let mut delete_token_hash = [0_u8; HASH_LEN];
                delete_token_hash.copy_from_slice(&body[code_end..code_end + HASH_LEN]);
                let expires_offset = code_end + HASH_LEN;
                let expires_at_ms = u64::from_be_bytes([
                    body[expires_offset],
                    body[expires_offset + 1],
                    body[expires_offset + 2],
                    body[expires_offset + 3],
                    body[expires_offset + 4],
                    body[expires_offset + 5],
                    body[expires_offset + 6],
                    body[expires_offset + 7],
                ]);
                let receive_ttl_offset = expires_offset + 8;
                let receive_ttl_ms = u64::from_be_bytes([
                    body[receive_ttl_offset],
                    body[receive_ttl_offset + 1],
                    body[receive_ttl_offset + 2],
                    body[receive_ttl_offset + 3],
                    body[receive_ttl_offset + 4],
                    body[receive_ttl_offset + 5],
                    body[receive_ttl_offset + 6],
                    body[receive_ttl_offset + 7],
                ]);
                let email_verified_at_offset = receive_ttl_offset + 8;
                let email_verified_at_ms = u64::from_be_bytes([
                    body[email_verified_at_offset],
                    body[email_verified_at_offset + 1],
                    body[email_verified_at_offset + 2],
                    body[email_verified_at_offset + 3],
                    body[email_verified_at_offset + 4],
                    body[email_verified_at_offset + 5],
                    body[email_verified_at_offset + 6],
                    body[email_verified_at_offset + 7],
                ]);
                let max_receives_offset = email_verified_at_offset + 8;
                let max_receives =
                    u16::from_be_bytes([body[max_receives_offset], body[max_receives_offset + 1]]);
                let payload_len_offset = max_receives_offset + 2;
                let payload_len = u32::from_be_bytes([
                    body[payload_len_offset],
                    body[payload_len_offset + 1],
                    body[payload_len_offset + 2],
                    body[payload_len_offset + 3],
                ]) as usize;
                let payload_offset = payload_len_offset + 4;
                if body.len() < payload_offset + payload_len {
                    continue;
                }
                let contact_email_offset = payload_offset + payload_len;
                let contact_email = if body.len() == contact_email_offset {
                    None
                } else {
                    let mut reader = Reader::new(&body[contact_email_offset..]);
                    match reader
                        .string()
                        .ok()
                        .and_then(|email| normalize_verification_email(&email).ok())
                    {
                        Some(email) if reader.is_done() => Some(email),
                        _ => None,
                    }
                };
                if expires_at_ms > now_ms {
                    index.insert(
                        code_hash,
                        PublishEntry {
                            publish_code: publish_code.to_string(),
                            delete_token_hash,
                            contact_email,
                            payload_offset: record_start
                                + RECORD_HEADER_LEN as u64
                                + payload_offset as u64,
                            payload_len: payload_len as u32,
                            expires_at_ms,
                            receive_ttl_ms,
                            email_verified_at_ms,
                            max_receives,
                            receives: 0,
                        },
                    );
                }
            }
            KIND_TOMBSTONE => {
                if body.len() == HASH_LEN {
                    let mut code_hash = [0_u8; HASH_LEN];
                    code_hash.copy_from_slice(&body);
                    index.remove(&code_hash);
                }
            }
            KIND_RECEIVE_COUNT => {
                if body.len() == HASH_LEN + 2 {
                    let mut code_hash = [0_u8; HASH_LEN];
                    code_hash.copy_from_slice(&body[0..HASH_LEN]);
                    let receives = u16::from_be_bytes([body[HASH_LEN], body[HASH_LEN + 1]]);
                    if let Some(entry) = index.get_mut(&code_hash) {
                        if receives >= entry.max_receives {
                            index.remove(&code_hash);
                        } else {
                            entry.receives = receives;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    file.seek(SeekFrom::End(0))?;
    Ok(index)
}

fn compacted_bytes_for_index(index: &HashMap<RecordHash, PublishEntry>) -> u64 {
    index
        .values()
        .map(|entry| {
            RECORD_HEADER_LEN as u64
                + put_record_payload_offset(&entry.publish_code) as u64
                + entry.payload_len as u64
        })
        .sum()
}

fn put_record_payload_offset(publish_code: &str) -> usize {
    HASH_LEN + 1 + publish_code.len() + HASH_LEN + 8 + 8 + 8 + 2 + 4
}

fn compact_shard(shard: &Shard) -> Result<CompactionReport, StoreError> {
    let bytes_before = shard.path.metadata().map(|m| m.len()).unwrap_or(0);
    let mut index = lock_store(&shard.index, "shard index")?;
    let mut file = lock_store(&shard.file, "shard file")?;

    if index.is_empty() {
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        return Ok(CompactionReport {
            shards_compacted: usize::from(bytes_before > 0),
            bytes_before,
            bytes_after: 0,
            live_records: 0,
        });
    }

    let tmp_path = compact_tmp_path(&shard.path);
    let mut compacted = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(&tmp_path)?;

    let mut rewritten = Vec::with_capacity(index.len());
    for (code_hash, entry) in index.iter() {
        let payload = read_payload(&mut file, entry.payload_offset, entry.payload_len)?;
        rewritten.push((*code_hash, entry.clone(), payload));
    }

    for (code_hash, entry, payload) in rewritten {
        let (payload_offset, payload_len) =
            append_put(&mut compacted, &code_hash, &entry, &payload)?;
        if let Some(current) = index.get_mut(&code_hash) {
            current.payload_offset = payload_offset;
            current.payload_len = payload_len;
        }
    }

    compacted.flush()?;
    compacted.sync_data()?;
    fs::rename(&tmp_path, &shard.path)?;
    *file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&shard.path)?;
    let bytes_after = shard.path.metadata().map(|m| m.len()).unwrap_or(0);
    Ok(CompactionReport {
        shards_compacted: 1,
        bytes_before,
        bytes_after,
        live_records: index.len(),
    })
}

fn compact_tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    tmp.set_extension("seg.compact");
    tmp
}

fn read_payload(file: &mut File, offset: u64, len: u32) -> Result<Vec<u8>, StoreError> {
    let mut payload = vec![0_u8; len as usize];
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut payload)?;
    file.seek(SeekFrom::End(0))?;
    Ok(payload)
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c9dc5_u32, |hash, byte| {
        hash.wrapping_mul(16777619) ^ *byte as u32
    })
}

fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn verification_abuse_block_expires_at_ms() -> u64 {
    unix_ms(
        SystemTime::now()
            .checked_add(VERIFICATION_ABUSE_BLOCK_TTL)
            .unwrap_or_else(SystemTime::now),
    )
}

pub fn bench_store(config: ServerConfig) -> Result<(), StoreError> {
    let requests = config.benchmark_requests;
    let payload_bytes = config.benchmark_payload_bytes;
    let store = PublishStore::open(config)?;
    let payload = benchmark_payload(payload_bytes);
    let request =
        protocol::encode_publish_request_with_email(900, 2, &payload, Some("bench@example.com"));
    let decoded = protocol::decode_request(&request, 16 * 1024)?;
    let start = Instant::now();
    let mut codes = Vec::with_capacity(requests);
    for _ in 0..requests {
        let response = store.handle(decoded.operation, &decoded.payload);
        if response[6] != 0 || response[7] != 0 {
            continue;
        }
        let mut reader = Reader::new(&response[14..]);
        reader.message_version()?;
        let code = reader.string()?;
        let _delete_token = reader.bytes()?;
        let _expires_at_ms = reader.u64()?;
        let _max_receives = reader.u16()?;
        let verification_url = reader.string()?;
        let (verify_code, token) = verification_query_parts(&verification_url)
            .ok_or_else(|| StoreError::Config("invalid verification URL".to_string()))?;
        if verify_code == code {
            let _ = store.verify_email(&verify_code, &token);
        }
        codes.push(code);
    }
    let create_elapsed = start.elapsed();
    let start = Instant::now();
    for code in &codes {
        let _ = store.receive(code);
    }
    let receive_elapsed = start.elapsed();
    println!(
        "store_create_rps={} store_receive_rps={} live={}",
        (codes.len() as f64 / create_elapsed.as_secs_f64()) as u64,
        (codes.len() as f64 / receive_elapsed.as_secs_f64()) as u64,
        store.stats().live
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn auto_routes_are_rebuilt_when_topology_members_join() {
        let temp = std::env::temp_dir().join(format!(
            "lockbox-publish-topology-auto-routes-{}",
            unix_ms(SystemTime::now())
        ));
        fs::create_dir_all(&temp).unwrap();
        let config = ServerConfig {
            state_dir: temp.clone(),
            topology_token: Some("token".to_string()),
            topology_servers: vec![TopologyServer {
                id: 0,
                url: "http://publish-0.example/v1/publish".to_string(),
                status: ServerStatus::Active,
                last_seen_ms: None,
            }],
            ..ServerConfig::default()
        };
        let store = PublishStore::open(config.clone()).unwrap();

        let topology = store
            .register_topology_server(TopologyRegistration {
                cluster_id: config.cluster_id.clone(),
                server_id: 2,
                server_url: "http://publish-2.example/v1/publish".to_string(),
                status: ServerStatus::Active,
                security_token: "token".to_string(),
            })
            .unwrap();

        let route_map: std::collections::HashSet<_> = topology
            .routes
            .iter()
            .map(|route| (route.owner_id, route.primary_id, route.failover_ids.clone()))
            .collect();
        assert_eq!(route_map.len(), 2);
        assert!(route_map.contains(&(0, 0, vec![2])));
        assert!(route_map.contains(&(2, 2, vec![0])));

        assert_eq!(topology.servers.len(), 2);
        assert_eq!(store.topology().routes.len(), 2);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn outbox_reloads_only_unacked_events() {
        let path = std::env::temp_dir().join(format!(
            "lockbox-publish-outbox-test-{}",
            unix_ms(SystemTime::now())
        ));
        append_outbox_event(&path, 1, b"one").unwrap();
        append_outbox_event(&path, 2, b"two").unwrap();
        append_outbox_ack(&path, 1).unwrap();

        let pending = load_outbox_pending(&path).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, 2);
        assert_eq!(pending[0].1, b"two");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn replication_sender_includes_server_token_header() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut stream);
            let body = protocol::encode_response(Operation::Replicate, Status::Success, &[]);
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            std::io::Write::write_all(&mut stream, headers.as_bytes()).unwrap();
            std::io::Write::write_all(&mut stream, &body).unwrap();
            String::from_utf8_lossy(&request).to_string()
        });

        let peer_url = format!("http://{addr}/v1/replicate");
        send_replication_request(&[peer_url], b"replication-body", Some("server-token")).unwrap();
        let request = server.join().unwrap();

        assert!(request.contains("x-lockbox-server-token: server-token\r\n"));
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
        let mut request = Vec::new();
        let mut buffer = [0_u8; 512];
        loop {
            let read = std::io::Read::read(stream, &mut buffer).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            let Some(header_end) = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|offset| offset + 4)
            else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if request.len() >= header_end + content_length {
                break;
            }
        }
        request
    }

    #[test]
    fn status_document_includes_replication_files() {
        let state_dir = std::env::temp_dir().join(format!(
            "lockbox-publish-status-test-{}",
            unix_ms(SystemTime::now())
        ));
        fs::create_dir_all(&state_dir).unwrap();
        let config = ServerConfig {
            state_dir: state_dir.clone(),
            replication_peer_urls: Vec::new(),
            ..ServerConfig::default()
        };
        let store = PublishStore::open(config).unwrap();
        store_replication_sequence(&state_dir.join("replication-origin-sequence"), 42).unwrap();
        append_outbox_event(&state_dir.join("replication-outbox.bin"), 42, b"event").unwrap();

        let status = store.status_document();
        assert_eq!(status.replication_last_sequence, 42);
        assert_eq!(status.replication_pending, 1);

        let _ = fs::remove_dir_all(state_dir);
    }

    #[test]
    fn replication_accepts_out_of_order_sequences() {
        let state_dir = std::env::temp_dir().join(format!(
            "lockbox-publish-out-of-order-test-{}",
            unix_ms(SystemTime::now())
        ));
        let store = PublishStore::open(ServerConfig {
            state_dir: state_dir.clone(),
            promoted_owner_ids: vec![0],
            ..ServerConfig::default()
        })
        .unwrap();

        let second = replication_put_event(2, "00123456789002", "second");
        let first = replication_put_event(1, "00123456789001", "first");

        assert!(store.apply_replication_event(second.clone()).unwrap());
        assert!(store.apply_replication_event(first.clone()).unwrap());
        assert!(!store.apply_replication_event(second).unwrap());

        assert_eq!(store.stats().live, 2);
        assert!(store.receive("00123456789001").is_ok());
        assert!(store.receive("00123456789002").is_ok());

        let _ = fs::remove_dir_all(state_dir);
    }

    #[test]
    fn delete_token_hashes_are_stable_across_replica_secrets() {
        let state_a = std::env::temp_dir().join(format!(
            "lockbox-publish-token-hash-a-{}",
            unix_ms(SystemTime::now())
        ));
        let state_b = std::env::temp_dir().join(format!(
            "lockbox-publish-token-hash-b-{}",
            unix_ms(SystemTime::now())
        ));
        let store_a = PublishStore::open(ServerConfig {
            state_dir: state_a.clone(),
            ..ServerConfig::default()
        })
        .unwrap();
        let store_b = PublishStore::open(ServerConfig {
            state_dir: state_b.clone(),
            ..ServerConfig::default()
        })
        .unwrap();

        assert_ne!(store_a.secret, store_b.secret);
        assert_eq!(
            store_a.delete_token_hash(b"replicated-delete-token"),
            store_b.delete_token_hash(b"replicated-delete-token")
        );
        assert_ne!(
            store_a.code_hash("00123456789012"),
            store_b.code_hash("00123456789012")
        );

        let _ = fs::remove_dir_all(state_a);
        let _ = fs::remove_dir_all(state_b);
    }

    #[test]
    fn publish_code_generation_rejects_persisted_bucket_collision() {
        let state_dir = temp_state_dir("persisted-collision");
        let store = PublishStore::open(ServerConfig {
            state_dir: state_dir.clone(),
            index_cache_entries: 0,
            ..ServerConfig::default()
        })
        .unwrap();
        let code = store.unique_code_from_value(0, 0, 123).unwrap().unwrap();
        let code_hash = store.code_hash(&code);
        let entry = PublishEntry {
            publish_code: code,
            delete_token_hash: [7_u8; HASH_LEN],
            contact_email: None,
            payload_offset: 0,
            payload_len: 0,
            expires_at_ms: unix_ms(SystemTime::now()) + 60_000,
            receive_ttl_ms: 60_000,
            email_verified_at_ms: unix_ms(SystemTime::now()),
            max_receives: 1,
            receives: 0,
        };

        store.append_bucket_put(&code_hash, &entry).unwrap();

        assert!(store.unique_code_from_value(0, 0, 123).unwrap().is_none());
        let _ = fs::remove_dir_all(state_dir);
    }

    #[cfg(unix)]
    #[test]
    fn server_secret_file_is_private_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let state_dir = temp_state_dir("server-secret-private");
        fs::create_dir_all(&state_dir).unwrap();

        let _ = load_or_create_secret(&state_dir).unwrap();

        let mode = fs::metadata(state_dir.join("server.secret"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(state_dir);
    }

    #[cfg(unix)]
    #[test]
    fn existing_server_secret_file_is_restricted_on_load() {
        use std::os::unix::fs::PermissionsExt;

        let state_dir = temp_state_dir("server-secret-existing-private");
        fs::create_dir_all(&state_dir).unwrap();
        let path = state_dir.join("server.secret");
        fs::write(&path, [3_u8; 32]).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        let secret = load_or_create_secret(&state_dir).unwrap();

        assert_eq!(secret, [3_u8; 32]);
        let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(state_dir);
    }

    fn temp_state_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lockbox-publish-{label}-{}-{}",
            std::process::id(),
            unix_ms(SystemTime::now())
        ))
    }

    fn replication_put_event(sequence: u64, publish_code: &str, label: &str) -> ReplicationEvent {
        ReplicationEvent {
            origin_server_id: 0,
            origin_epoch: 1,
            origin_sequence: sequence,
            kind: ReplicationEventKind::PutPublish {
                publish_code: publish_code.to_string(),
                delete_token_hash: [sequence as u8; HASH_LEN].to_vec(),
                payload: payload::encode_contact_publish(
                    &format!("{label}@example.com"),
                    b"public-key-material",
                    b"signing-public-key-material",
                    &[1_u8; 32],
                    &[2_u8; 24],
                    1,
                    2,
                ),
                contact_email: Some(format!("{label}@example.com")),
                expires_at_unix_ms: unix_ms(SystemTime::now()) + 60_000,
                receive_ttl_ms: 60_000,
                email_verified_at_unix_ms: unix_ms(SystemTime::now()),
                max_receives: 2,
                receives: 0,
            },
        }
    }
}

fn benchmark_payload(target_bytes: usize) -> Vec<u8> {
    let key_len = target_bytes.saturating_sub(112).clamp(32, 4096);
    let public_key = vec![42_u8; key_len];
    payload::encode_contact_publish(
        "bench@example.com",
        &public_key,
        b"signing-public-key-material",
        &[7_u8; 32],
        &[9_u8; 24],
        1,
        2,
    )
}
