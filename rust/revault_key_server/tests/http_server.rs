use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use revault_key_server::server::run_listener;
use revault_key_server::store::{PublishStore, ServerConfig};
use revault_publish_protocol::{
    decode_topology, encode_replication_request, encode_topology_registration, protocol,
    sign_replication_event, ReplicationEvent, ReplicationEventKind, ReplicationRequest,
    ServerStatus, TopologyRegistration,
};

#[test]
fn axum_server_rate_limits_topology_and_allows_server_token_header() {
    let (_guard, mut config) = temp_server_config("axum-topology-rate-limit");
    config.topology_token = Some("server-token".to_string());
    config.rate_limit_per_minute = 60;
    config.rate_limit_burst = 1;
    let addr = spawn_server(config);

    let first = get_request(addr, "/v1/topology", "");
    decode_topology(http_body(&first)).unwrap();

    let second = get_request(addr, "/v1/topology", "");
    let response = protocol::decode_response(http_body(&second), 1024).unwrap();
    assert_eq!(response.status, protocol::Status::RateLimited);

    let authenticated = get_request(
        addr,
        "/v1/topology",
        "X-Lockbox-Server-Token: server-token\r\n",
    );
    decode_topology(http_body(&authenticated)).unwrap();
}

#[test]
fn axum_server_token_header_does_not_bypass_publish_rate_limit() {
    let (_guard, mut config) = temp_server_config("axum-publish-rate-limit");
    config.topology_token = Some("server-token".to_string());
    config.rate_limit_per_minute = 60;
    config.rate_limit_burst = 1;
    let addr = spawn_server(config);
    let body = b"not-a-protocol-request";

    let first = post_request(
        addr,
        "/v1/publish",
        "X-Lockbox-Server-Token: server-token\r\n",
        body,
    );
    let response = protocol::decode_response(http_body(&first), 1024).unwrap();
    assert_eq!(response.status, protocol::Status::MalformedRequest);

    let second = post_request(
        addr,
        "/v1/publish",
        "X-Lockbox-Server-Token: server-token\r\n",
        body,
    );
    let response = protocol::decode_response(http_body(&second), 1024).unwrap();
    assert_eq!(response.status, protocol::Status::RateLimited);
}

#[test]
fn axum_server_replicate_accepts_authenticated_replication_request() {
    let (_guard, mut config) = temp_server_config("axum-replicate");
    config.server_id = 1;
    config.topology_token = Some("server-token".to_string());
    config.replication_token = Some("peer-secret".to_string());
    let server = spawn_server_with_store(config);
    let event = ReplicationEvent {
        origin_server_id: 0,
        origin_epoch: 2,
        origin_sequence: 1,
        kind: ReplicationEventKind::RateLimitBlock {
            client_ip: "203.0.113.25".to_string(),
            expires_at_unix_ms: unix_ms(SystemTime::now() + Duration::from_secs(60)),
        },
    };
    let body = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    });

    let response = post_request(
        server.addr,
        "/v1/replicate",
        "X-Lockbox-Server-Token: server-token\r\n",
        &body,
    );

    let response = protocol::decode_response(http_body(&response), 1024).unwrap();
    assert_eq!(response.status, protocol::Status::Success);
    assert!(server
        .store
        .is_rate_limit_blocked(Some(std::net::IpAddr::from([203, 0, 113, 25]))));
}

#[test]
fn axum_server_topology_register_accepts_authenticated_registration() {
    let (_guard, mut config) = temp_server_config("axum-topology-register");
    config.topology_token = Some("server-token".to_string());
    let server = spawn_server_with_store(config);
    let body = encode_topology_registration(&TopologyRegistration {
        cluster_id: "default".to_string(),
        server_id: 7,
        server_url: "http://peer.example/v1/publish".to_string(),
        status: ServerStatus::Active,
        security_token: "server-token".to_string(),
    })
    .unwrap();

    let response = post_request(
        server.addr,
        "/v1/topology/register",
        "X-Lockbox-Server-Token: server-token\r\n",
        &body,
    );

    let topology = decode_topology(http_body(&response)).unwrap();
    assert!(topology.servers.iter().any(|server| server.id == 7));
}

#[test]
fn axum_server_rejects_payloads_over_configured_limit() {
    let (_guard, mut config) = temp_server_config("axum-payload-too-large");
    config.max_payload_bytes = 8;
    let server = spawn_server_with_store(config);
    let body = vec![0_u8; 8 + 129];

    let response = post_request(server.addr, "/v1/publish", "", &body);

    if response.starts_with(b"HTTP/1.1 413") {
        return;
    }
    let response = protocol::decode_response(http_body(&response), 1024).unwrap();
    assert_eq!(response.status, protocol::Status::PayloadTooLarge);
}

#[test]
fn axum_server_verify_page_reports_missing_token_as_html_error() {
    let (_guard, config) = temp_server_config("axum-verify-missing-token");
    let server = spawn_server_with_store(config);

    let response = get_request(server.addr, "/v1/verify?code=missing", "");

    assert!(response.starts_with(b"HTTP/1.1 400 Bad Request"));
    assert!(String::from_utf8_lossy(&response).contains("text/html"));
    assert!(String::from_utf8_lossy(&response).contains("Verification failed"));
}

struct TestServer {
    addr: SocketAddr,
    store: Arc<PublishStore>,
}

fn spawn_server(config: ServerConfig) -> SocketAddr {
    spawn_server_with_store(config).addr
}

fn spawn_server_with_store(config: ServerConfig) -> TestServer {
    let store = Arc::new(PublishStore::open(config).unwrap());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server_store = Arc::clone(&store);
    thread::spawn(move || {
        let _ = run_listener(listener, server_store);
    });
    thread::sleep(Duration::from_millis(50));
    TestServer { addr, store }
}

fn get_request(addr: SocketAddr, target: &str, headers: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).unwrap();
    let request =
        format!("GET {target} HTTP/1.1\r\nHost: {addr}\r\n{headers}Connection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    response
}

fn post_request(addr: SocketAddr, target: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).unwrap();
    let request = format!(
        "POST {target} HTTP/1.1\r\nHost: {addr}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(request.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    response
}

fn http_body(response: &[u8]) -> &[u8] {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("HTTP response has headers");
    &response[header_end + 4..]
}

fn temp_server_config(label: &str) -> (TempDir, ServerConfig) {
    let guard = TempDir::new(label);
    let config = ServerConfig {
        state_dir: guard.path.clone(),
        ..ServerConfig::default()
    };
    (guard, config)
}

fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "revault-key-server-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
