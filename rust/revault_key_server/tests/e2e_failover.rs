use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use revault_key_server::server::run_listener;
use revault_key_server::store::{PublishStore, ServerConfig};
use revault_publish_protocol::protocol::{self, Operation, Status};
use revault_publish_protocol::{
    decode_contact_publish, encode_contact_publish, encode_replication_request, ClientError,
    HttpTransport, PublishClient, PublishClientPool, ReplicationEvent, ReplicationEventKind,
    ReplicationRequest, ServerStatus, TopologyRoute, TopologyServer, Transport,
};

const REPLICATION_TOKEN: &str = "e2e-replication-token";

#[test]
#[ignore = "requires local TCP sockets; run explicitly on a host with loopback networking"]
fn two_server_failover_receive_delete_and_edge_cases() {
    if !has_loopback_sockets() {
        eprintln!("skipping local-socket e2e test in restricted environment");
        return;
    }
    let cluster = TwoServerCluster::start("route-failover", PeerMode::BothDirections);
    let primary = PublishClient::new(&cluster.primary.publish_url())
        .unwrap()
        .with_timeout(Duration::from_millis(250));
    let payload = contact_payload("route-failover");
    let published = primary
        .publish_payload_with_email(60, 3, &payload, Some("route-failover@example.com"))
        .unwrap();
    cluster.verify_publish(&published);
    assert!(published.publish_code.starts_with('0'));
    wait_until("replication to standby", Duration::from_secs(10), || {
        cluster.standby.store.stats().live >= 1
    });

    let failover_pool = cluster.pool_with_dead_primary();
    let received = failover_pool.receive(&published.publish_code).unwrap();
    assert_eq!(received.payload, payload);
    assert_eq!(
        decode_contact_publish(&received.payload).unwrap().identity,
        "route-failover@example.com"
    );

    let bad_code = failover_pool.receive("x-not-a-publish").unwrap_err();
    assert_server_error(bad_code, Status::PublishNotFound);
    let bad_token = failover_pool
        .delete(&published.publish_code, b"wrong-delete-token")
        .unwrap_err();
    assert_server_error(bad_token, Status::DeleteTokenInvalid);

    assert!(failover_pool
        .delete(&published.publish_code, &published.delete_token)
        .unwrap());
    wait_until(
        "standby delete tombstone replicated to primary",
        Duration::from_secs(10),
        || primary.receive(&published.publish_code).is_err(),
    );
    assert_server_error(
        primary.receive(&published.publish_code).unwrap_err(),
        Status::PublishNotFound,
    );

    let single = primary
        .publish_payload_with_email(
            60,
            1,
            &contact_payload("single-use"),
            Some("single-use@example.com"),
        )
        .unwrap();
    cluster.verify_publish(&single);
    let mut first_receive = None;
    wait_until(
        "single-use publish to become receivable from standby",
        Duration::from_secs(10),
        || match failover_pool.receive(&single.publish_code) {
            Ok(received) => {
                first_receive = Some(received);
                true
            }
            Err(ClientError::Server {
                status: Status::PublishNotFound | Status::EmailUnverified,
                ..
            }) => false,
            Err(error) => panic!("unexpected failover receive error: {error}"),
        },
    );
    assert_eq!(
        first_receive
            .expect("wait completed without a receive")
            .payload,
        contact_payload("single-use")
    );
    assert_server_error(
        failover_pool.receive(&single.publish_code).unwrap_err(),
        Status::PublishNotFound,
    );
    wait_until(
        "single-use standby tombstone replicated to primary",
        Duration::from_secs(10),
        || primary.receive(&single.publish_code).is_err(),
    );
}

#[test]
#[ignore = "requires local TCP sockets; run explicitly on a host with loopback networking"]
fn resync_recovers_cold_standby_after_missed_replication() {
    if !has_loopback_sockets() {
        eprintln!("skipping local-socket e2e test in restricted environment");
        return;
    }
    let cluster = TwoServerCluster::start("cold-standby", PeerMode::NoAutomaticPeers);
    let primary = PublishClient::new(&cluster.primary.publish_url())
        .unwrap()
        .with_timeout(Duration::from_millis(250));
    let standby = PublishClient::new(&cluster.standby.publish_url())
        .unwrap()
        .with_timeout(Duration::from_millis(250));

    let mut published = Vec::new();
    for index in 0..8 {
        published.push(
            primary
                .publish_payload_with_email(
                    60,
                    2,
                    &contact_payload(&format!("resync-{index}")),
                    Some(&format!("resync-{index}@example.com")),
                )
                .unwrap(),
        );
        cluster.verify_publish(published.last().unwrap());
    }
    assert_server_error(
        standby.receive(&published[0].publish_code).unwrap_err(),
        Status::PublishNotFound,
    );

    let sent = cluster
        .primary
        .store
        .resync_peer(&cluster.standby.replicate_url())
        .unwrap();
    assert_eq!(sent, published.len());
    wait_until("resync applied on standby", Duration::from_secs(10), || {
        cluster.standby.store.stats().live >= published.len()
    });
    for published_payload in &published {
        assert_eq!(
            standby
                .receive(&published_payload.publish_code)
                .unwrap()
                .payload_type as u16,
            1
        );
    }

    let sent_again = cluster
        .primary
        .store
        .resync_peer(&cluster.standby.replicate_url())
        .unwrap();
    assert_eq!(sent_again, published.len());
    assert_eq!(cluster.standby.store.stats().live, published.len());

    let request = encode_replication_request(&ReplicationRequest {
        authentication: b"invalid".to_vec(),
        event: ReplicationEvent {
            origin_server_id: 0,
            origin_epoch: 1,
            origin_sequence: 999,
            kind: ReplicationEventKind::Tombstone {
                publish_code: published[0].publish_code.clone(),
            },
        },
    });
    let response = HttpTransport::new(&cluster.standby.replicate_url())
        .unwrap()
        .post_binary(&request)
        .unwrap();
    let envelope = protocol::decode_response(&response, 1024).unwrap();
    assert_eq!(envelope.operation, Operation::Replicate);
    assert_eq!(envelope.status, Status::ReplicationUnauthorized);
}

#[test]
#[ignore = "requires local TCP sockets and performs a concurrent failover load test"]
fn heavy_failover_recovery_under_load() {
    if !has_loopback_sockets() {
        eprintln!("skipping local-socket e2e test in restricted environment");
        return;
    }
    let flows = std::env::var("LOCKBOX_SHARE_E2E_FLOWS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50_000);
    let workers = std::env::var("LOCKBOX_SHARE_E2E_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(default_heavy_workers)
        .max(1);

    let cluster = TwoServerCluster::start("heavy-recover", PeerMode::StandbyAppearsLate);
    let created = Arc::new(AtomicUsize::new(0));
    let received = Arc::new(AtomicUsize::new(0));
    let monitor = ProgressMonitor::start(
        flows,
        Arc::clone(&created),
        Arc::clone(&received),
        Arc::clone(&cluster.primary.store),
        Arc::clone(&cluster.standby.store),
    );
    let primary = PublishClient::new(&cluster.primary.publish_url())
        .unwrap()
        .with_timeout(Duration::from_millis(500))
        .with_retry_policy(100, Duration::from_millis(5), Duration::from_millis(250));
    let primary_store = Arc::clone(&cluster.primary.store);
    let codes = Arc::new(Mutex::new(Vec::with_capacity(flows)));
    let create_start = Instant::now();
    run_parallel(workers, flows, {
        let primary = primary.clone();
        let primary_store = Arc::clone(&primary_store);
        let codes = Arc::clone(&codes);
        let created = Arc::clone(&created);
        move |index| {
            let published = primary
                .publish_payload_with_email(
                    600,
                    64,
                    &contact_payload(&format!("heavy-{index}")),
                    Some(&format!("heavy-{index}@example.com")),
                )
                .unwrap();
            verify_publish_on_store(&primary_store, &published);
            codes.lock().unwrap().push(published.publish_code);
            created.fetch_add(1, Ordering::Relaxed);
        }
    });
    let create_elapsed = create_start.elapsed();
    let codes = Arc::try_unwrap(codes).unwrap().into_inner().unwrap();
    assert_eq!(codes.len(), flows);

    cluster.start_late_standby();
    let recover_start = Instant::now();
    wait_until("late standby caught up", Duration::from_secs(180), || {
        cluster.standby.store.stats().live >= flows
    });
    let recover_elapsed = recover_start.elapsed();

    let failover_pool = Arc::new(cluster.pool_with_dead_primary());
    let receive_start = Instant::now();
    run_parallel(workers, codes.len(), {
        let codes = Arc::new(codes);
        let failover_pool = Arc::clone(&failover_pool);
        let received = Arc::clone(&received);
        move |index| {
            let received_publish = failover_pool.receive(&codes[index]).unwrap();
            assert_eq!(received_publish.payload_type as u16, 1);
            received.fetch_add(1, Ordering::Relaxed);
        }
    });
    let receive_elapsed = receive_start.elapsed();
    assert_eq!(received.load(Ordering::Relaxed), flows);
    assert!(cluster.standby.store.stats().received >= flows as u64);
    monitor.stop();
    eprintln!(
        "heavy_failover flows={flows} workers={workers} create_rps={} recover_ms={} receive_rps={}",
        rps(flows, create_elapsed),
        recover_elapsed.as_millis(),
        rps(flows, receive_elapsed)
    );
}

#[derive(Clone, Copy)]
enum PeerMode {
    BothDirections,
    NoAutomaticPeers,
    StandbyAppearsLate,
}

struct TwoServerCluster {
    _guard: TempDir,
    primary: RunningServer,
    standby: RunningServer,
    late_standby: Mutex<Option<ServerConfig>>,
    topology_servers: Vec<TopologyServer>,
    topology_routes: Vec<TopologyRoute>,
}

impl TwoServerCluster {
    fn start(name: &str, peer_mode: PeerMode) -> Self {
        let guard = TempDir::new(name);
        let primary_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let standby_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let primary_addr = primary_listener.local_addr().unwrap();
        let standby_addr = standby_listener.local_addr().unwrap();
        let primary_url = publish_url(primary_addr);
        let standby_url = publish_url(standby_addr);
        let primary_replicate_url = replicate_url(primary_addr);
        let standby_replicate_url = replicate_url(standby_addr);
        let topology_servers = vec![
            TopologyServer {
                id: 0,
                url: primary_url.clone(),
                status: ServerStatus::Active,
                last_seen_ms: None,
            },
            TopologyServer {
                id: 1,
                url: standby_url.clone(),
                status: ServerStatus::Promoted,
                last_seen_ms: None,
            },
        ];
        let topology_routes = vec![
            TopologyRoute {
                owner_id: 0,
                primary_id: 0,
                failover_ids: vec![1],
            },
            TopologyRoute {
                owner_id: 1,
                primary_id: 1,
                failover_ids: vec![0],
            },
        ];

        let mut primary_config = config(
            0,
            primary_addr,
            guard.path.join("primary"),
            topology_servers.clone(),
            topology_routes.clone(),
            Vec::new(),
            Vec::new(),
        );
        let mut standby_config = config(
            1,
            standby_addr,
            guard.path.join("standby"),
            topology_servers.clone(),
            topology_routes.clone(),
            vec![0],
            Vec::new(),
        );
        match peer_mode {
            PeerMode::BothDirections => {
                primary_config.replication_peer_urls = vec![standby_replicate_url.clone()];
                standby_config.replication_peer_urls = vec![primary_replicate_url.clone()];
            }
            PeerMode::NoAutomaticPeers => {}
            PeerMode::StandbyAppearsLate => {
                primary_config.replication_peer_urls = vec![standby_replicate_url.clone()];
                standby_config.replication_peer_urls = vec![primary_replicate_url.clone()];
            }
        }

        let primary = RunningServer::start(primary_listener, primary_config);
        let (standby, late_standby) = match peer_mode {
            PeerMode::StandbyAppearsLate => (
                RunningServer::placeholder(standby_addr, standby_config.clone()),
                {
                    drop(standby_listener);
                    Some(standby_config)
                },
            ),
            _ => (RunningServer::start(standby_listener, standby_config), None),
        };

        Self {
            _guard: guard,
            primary,
            standby,
            late_standby: Mutex::new(late_standby),
            topology_servers,
            topology_routes,
        }
    }

    fn start_late_standby(&self) {
        let Some(config) = self.late_standby.lock().unwrap().take() else {
            return;
        };
        self.standby.start_placeholder(config);
    }

    fn pool_with_dead_primary(&self) -> PublishClientPool {
        let mut servers = self.topology_servers.clone();
        servers[0].url = unused_publish_url();
        let topology = revault_publish_protocol::ClusterTopology {
            cluster_id: "e2e".to_string(),
            version: 1,
            servers,
            routes: self.topology_routes.clone(),
        };
        PublishClientPool::from_topology(&topology)
            .unwrap()
            .with_timeout(Duration::from_millis(150))
            .with_retry_policy(100, Duration::from_millis(5), Duration::from_millis(250))
    }

    fn verify_publish(&self, published: &revault_publish_protocol::PublishResult) {
        let owner = if published.publish_code.starts_with('0') {
            self.primary.store.as_ref()
        } else {
            self.standby.store.as_ref()
        };
        verify_publish_on_store(owner, published);
    }
}

fn verify_publish_on_store(
    store: &PublishStore,
    published: &revault_publish_protocol::PublishResult,
) {
    let verification_url = published
        .verification_url
        .as_deref()
        .expect("publish should include verification URL");
    let (code, token) = verification_query_parts(verification_url);
    assert_eq!(code, published.publish_code);
    assert!(store.verify_email(&code, &token).success);
}

fn verification_query_parts(url: &str) -> (String, String) {
    let query = url.split_once('?').unwrap().1;
    let mut code = None;
    let mut token = None;
    for part in query.split('&') {
        let (key, value) = part.split_once('=').unwrap();
        match key {
            "code" => code = Some(value.to_string()),
            "token" => token = Some(value.to_string()),
            _ => {}
        }
    }
    (code.unwrap(), token.unwrap())
}

struct RunningServer {
    addr: SocketAddr,
    store: Arc<PublishStore>,
}

impl RunningServer {
    fn start(listener: TcpListener, config: ServerConfig) -> Self {
        let addr = listener.local_addr().unwrap();
        let store = Arc::new(PublishStore::open(config).unwrap());
        let server_store = Arc::clone(&store);
        thread::spawn(move || {
            let _ = run_listener(listener, server_store);
        });
        wait_for_http(addr);
        Self { addr, store }
    }

    fn placeholder(addr: SocketAddr, config: ServerConfig) -> Self {
        let store = Arc::new(PublishStore::open(config.clone()).unwrap());
        Self { addr, store }
    }

    fn start_placeholder(&self, config: ServerConfig) {
        let listener = TcpListener::bind(self.addr).unwrap();
        let server_store = Arc::clone(&self.store);
        thread::spawn(move || {
            let _ = run_listener(listener, server_store);
        });
        let _ = config;
        wait_for_http(self.addr);
    }

    fn publish_url(&self) -> String {
        publish_url(self.addr)
    }

    fn replicate_url(&self) -> String {
        replicate_url(self.addr)
    }
}

fn config(
    server_id: u8,
    addr: SocketAddr,
    state_dir: PathBuf,
    topology_servers: Vec<TopologyServer>,
    topology_routes: Vec<TopologyRoute>,
    promoted_owner_ids: Vec<u8>,
    replication_peer_urls: Vec<String>,
) -> ServerConfig {
    ServerConfig {
        bind_addr: addr.to_string(),
        state_dir,
        server_id,
        cluster_id: "e2e".to_string(),
        public_url: Some(publish_url(addr)),
        topology_version: 1,
        topology_servers,
        topology_routes,
        replication_token: Some(REPLICATION_TOKEN.to_string()),
        replication_peer_urls,
        promoted_owner_ids,
        max_payload_bytes: 8 * 1024,
        verification_ttl: Duration::from_secs(1800),
        default_receive_ttl: Duration::from_secs(600),
        max_receive_ttl: Duration::from_secs(600),
        shard_count: 4,
        developer_mode: true,
        benchmark_requests: 0,
        benchmark_payload_bytes: 0,
        benchmark_concurrency: 0,
        benchmark_preload_published_payloads: 0,
        max_receives_per_publish: 64,
        compact_min_bytes: 1024 * 1024,
        index_cache_entries: 100_000,
        rate_limit_per_minute: 0,
        rate_limit_burst: 1_000,
        verification_email_rate_limit_per_hour: 0,
        verification_email_ip_rate_limit_per_hour: 0,
        ..ServerConfig::default()
    }
}

fn run_parallel<F>(workers: usize, jobs: usize, f: F)
where
    F: Fn(usize) + Send + Sync + 'static,
{
    let f = Arc::new(f);
    let next = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut handles = Vec::new();
    for worker_id in 0..workers {
        let f = Arc::clone(&f);
        let next = Arc::clone(&next);
        handles.push((
            worker_id,
            thread::spawn(move || loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if index >= jobs {
                    break;
                }
                f(index);
            }),
        ));
    }
    for (worker_id, handle) in handles {
        if let Err(panic) = handle.join() {
            if let Some(message) = panic.downcast_ref::<&str>() {
                panic!("worker {worker_id} panicked: {message}");
            }
            if let Some(message) = panic.downcast_ref::<String>() {
                panic!("worker {worker_id} panicked: {message}");
            }
            panic!("worker {worker_id} panicked with non-string payload");
        }
    }
}

struct ProgressMonitor {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ProgressMonitor {
    fn start(
        flows: usize,
        created: Arc<AtomicUsize>,
        received: Arc<AtomicUsize>,
        primary: Arc<PublishStore>,
        standby: Arc<PublishStore>,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let started = Instant::now();
            while !thread_stop.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));
                let primary_stats = primary.stats();
                let standby_stats = standby.stats();
                let elapsed = started.elapsed().as_secs().max(1);
                eprintln!(
                    "heavy_failover progress elapsed={}s target={} created={} received={} \
                     primary_live={} primary_pending={} standby_live={} standby_pending={} \
                     create_rate={} receive_rate={}",
                    elapsed,
                    flows,
                    created.load(Ordering::Relaxed),
                    received.load(Ordering::Relaxed),
                    primary_stats.live,
                    primary_stats.replication_pending,
                    standby_stats.live,
                    standby_stats.replication_pending,
                    created.load(Ordering::Relaxed) as u64 / elapsed,
                    received.load(Ordering::Relaxed) as u64 / elapsed
                );
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }

    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ProgressMonitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn default_heavy_workers() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .saturating_mul(8)
        .clamp(32, 128)
}

fn has_loopback_sockets() -> bool {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => false,
        Err(error) => panic!("unable to bind 127.0.0.1:0 for local e2e server: {error}"),
    }
}

fn contact_payload(label: &str) -> Vec<u8> {
    encode_contact_publish(
        &format!("{label}@example.com"),
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    )
}

fn wait_until(label: &str, timeout: Duration, mut predicate: impl FnMut() -> bool) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if predicate() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {label}");
}

fn rps(count: usize, elapsed: Duration) -> u64 {
    let seconds = elapsed.as_secs_f64();
    if seconds == 0.0 {
        return count as u64;
    }
    (count as f64 / seconds) as u64
}

fn wait_for_http(addr: SocketAddr) {
    wait_until("server listener", Duration::from_secs(5), || {
        std::net::TcpStream::connect(addr).is_ok()
    });
}

fn assert_server_error(error: ClientError, status: Status) {
    match error {
        ClientError::Server { status: actual, .. } => assert_eq!(actual, status),
        other => panic!("expected {status:?} server error, got {other:?}"),
    }
}

fn publish_url(addr: SocketAddr) -> String {
    format!("http://{addr}/v1/publish")
}

fn replicate_url(addr: SocketAddr) -> String {
    format!("http://{addr}/v1/replicate")
}

fn unused_publish_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    publish_url(addr)
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "revault-publish-e2e-{name}-{}-{:?}",
            std::process::id(),
            thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
