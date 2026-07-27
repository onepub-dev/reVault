use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::server_log::log_server_event;
use crate::store::{PublishStore, ServerConfig};
use axum::body::Bytes;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use revault_publish_protocol::payload;
use revault_publish_protocol::protocol::{self, Operation, Status};
use revault_publish_protocol::status;
use revault_publish_protocol::topology;
use serde::Deserialize;

const MAX_WIRE_OVERHEAD: usize = 128;
const CLUSTER_RATE_LIMIT_BLOCK_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Returns the run server.
pub fn run_server(bind: &str, store: Arc<PublishStore>) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    run_listener(listener, store)
}

/// Returns the run listener.
pub fn run_listener(listener: TcpListener, store: Arc<PublishStore>) -> std::io::Result<()> {
    let local_addr = listener.local_addr()?;
    log_server_event(format!("revault_key_server listening on {local_addr}"));
    start_background_maintenance(&store);
    listener.set_nonblocking(true)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(std::io::Error::other)?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        axum::serve(
            listener,
            make_app(store).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(std::io::Error::other)
    })
}

/// Returns the local addr.
pub fn local_addr(listener: &TcpListener) -> std::io::Result<SocketAddr> {
    listener.local_addr()
}

fn start_background_maintenance(store: &Arc<PublishStore>) {
    let purge_store = Arc::clone(store);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        purge_store.purge_expired();
    });
    let compact_store = Arc::clone(store);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));
        if let Err(err) = compact_store.compact_if_needed() {
            log_server_event(format!("compaction failed: {err}"));
        }
    });
}

#[derive(Clone)]
struct AppState {
    store: Arc<PublishStore>,
    limiter: Arc<RateLimiter>,
}

fn make_app(store: Arc<PublishStore>) -> Router {
    let body_limit = store.max_payload_bytes() + MAX_WIRE_OVERHEAD;
    let limiter = Arc::new(RateLimiter::new(
        store.rate_limit_per_minute(),
        store.rate_limit_burst(),
    ));
    Router::new()
        .route("/v1/publish", post(publish_handler))
        .route("/v1/replicate", post(replicate_handler))
        .route("/v1/topology/register", post(topology_register_handler))
        .route("/v1/topology", get(topology_handler))
        .route("/v1/status", get(status_handler))
        .route("/v1/verify", get(verify_handler))
        .layer(DefaultBodyLimit::max(body_limit))
        .with_state(AppState { store, limiter })
}

async fn publish_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    binary_post_handler(
        state,
        headers,
        Some(peer.ip()),
        BinaryEndpoint::Publish,
        body,
    )
    .await
}

async fn replicate_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    binary_post_handler(
        state,
        headers,
        Some(peer.ip()),
        BinaryEndpoint::Replicate,
        body,
    )
    .await
}

async fn topology_register_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    binary_post_handler(
        state,
        headers,
        Some(peer.ip()),
        BinaryEndpoint::TopologyRegister,
        body,
    )
    .await
}

async fn topology_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !request_has_server_token_headers(&headers, &state.store)
        && !allow_anonymous_request(&state.store, &state.limiter, Some(peer.ip()))
    {
        return rate_limited_response();
    }
    match topology::encode_topology(&state.store.topology()) {
        Ok(body) => binary_response(StatusCode::OK, body),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn status_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !request_has_server_token_headers(&headers, &state.store)
        && !allow_anonymous_request(&state.store, &state.limiter, Some(peer.ip()))
    {
        return rate_limited_response();
    }
    binary_response(
        StatusCode::OK,
        status::encode_status(&state.store.status_document()),
    )
}

#[derive(Deserialize)]
struct VerifyQuery {
    code: Option<String>,
    token: Option<String>,
}

async fn verify_handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(query): Query<VerifyQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_has_server_token_headers(&headers, &state.store)
        && !allow_anonymous_request(&state.store, &state.limiter, Some(peer.ip()))
    {
        return rate_limited_response();
    }
    let page = match (query.code, query.token) {
        (Some(code), Some(token)) => state.store.verify_email(&code, &token),
        _ => verify_error(
            "Verification failed",
            "The verification link is missing its token.",
        ),
    };
    let status = if page.success {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (status, Html(render_verify_page(&page))).into_response()
}

#[derive(Clone, Copy)]
enum BinaryEndpoint {
    Publish,
    Replicate,
    TopologyRegister,
}

async fn binary_post_handler(
    state: AppState,
    headers: HeaderMap,
    peer_ip: Option<IpAddr>,
    endpoint: BinaryEndpoint,
    body: Bytes,
) -> Response {
    let inter_server_authenticated = matches!(
        endpoint,
        BinaryEndpoint::Replicate | BinaryEndpoint::TopologyRegister
    ) && request_has_server_token_headers(&headers, &state.store);
    if !inter_server_authenticated
        && !allow_anonymous_request(&state.store, &state.limiter, peer_ip)
    {
        return rate_limited_response();
    }
    if body.len() > state.store.max_payload_bytes() + MAX_WIRE_OVERHEAD {
        return protocol_response(protocol::encode_error(
            Operation::Publish,
            Status::PayloadTooLarge,
            "too large",
        ));
    }
    let store = Arc::clone(&state.store);
    let body = body.to_vec();
    let response = tokio::task::spawn_blocking(move || match endpoint {
        BinaryEndpoint::TopologyRegister => match store.handle_topology_registration(&body) {
            Ok(response) => response,
            Err(err) => protocol::encode_error(
                Operation::Publish,
                Status::StoreUnavailable,
                &err.to_string(),
            ),
        },
        BinaryEndpoint::Publish | BinaryEndpoint::Replicate => {
            match protocol::decode_request(&body, store.max_payload_bytes() + MAX_WIRE_OVERHEAD) {
                Ok(request) => {
                    let _ = request.flags;
                    let replicate_endpoint = matches!(endpoint, BinaryEndpoint::Replicate);
                    if replicate_endpoint && request.operation != Operation::Replicate {
                        protocol::encode_error(
                            request.operation,
                            Status::UnknownOperation,
                            "replication endpoint accepts only replication operations",
                        )
                    } else if !replicate_endpoint && request.operation == Operation::Replicate {
                        protocol::encode_error(
                            request.operation,
                            Status::UnknownOperation,
                            "publish endpoint does not accept replication operations",
                        )
                    } else {
                        store.handle_with_peer(request.operation, &request.payload, peer_ip)
                    }
                }
                Err(err) => protocol::encode_error(
                    Operation::Publish,
                    Status::MalformedRequest,
                    &err.to_string(),
                ),
            }
        }
    })
    .await
    .unwrap_or_else(|err| {
        protocol::encode_error(
            Operation::Publish,
            Status::StoreUnavailable,
            &err.to_string(),
        )
    });
    protocol_response(response)
}

fn protocol_response(body: Vec<u8>) -> Response {
    binary_response(StatusCode::OK, body)
}

fn rate_limited_response() -> Response {
    protocol_response(protocol::encode_error(
        Operation::Publish,
        Status::RateLimited,
        "rate limited",
    ))
}

fn binary_response(status: StatusCode, body: Vec<u8>) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    )
        .into_response()
}

fn request_has_server_token_headers(headers: &HeaderMap, store: &PublishStore) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_token_header)
        .or_else(|| header_string(headers, "x-lockbox-server-token"))
        .or_else(|| header_string(headers, "x-topology-token"))
        .as_deref()
        .is_some_and(|token| store.topology_token_matches(token))
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bearer_token_header(value: &str) -> Option<String> {
    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// Represents rate limiter.
pub struct RateLimiter {
    per_minute: u32,
    burst: u32,
    clients: Mutex<HashMap<IpAddr, ClientBucket>>,
}

struct ClientBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(per_minute: u32, burst: u32) -> Self {
        Self {
            per_minute,
            burst: burst.max(1),
            clients: Mutex::new(HashMap::new()),
        }
    }

    fn allow(&self, peer_ip: Option<IpAddr>) -> bool {
        if self.per_minute == 0 {
            return true;
        }
        let Some(peer_ip) = peer_ip else {
            return false;
        };
        let now = Instant::now();
        let refill_per_second = self.per_minute as f64 / 60.0;
        let Ok(mut clients) = self.clients.lock() else {
            return false;
        };
        let bucket = clients.entry(peer_ip).or_insert(ClientBucket {
            tokens: self.burst as f64,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * refill_per_second).min(self.burst as f64);
        bucket.last_refill = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

fn allow_anonymous_request(
    store: &PublishStore,
    limiter: &RateLimiter,
    peer_ip: Option<IpAddr>,
) -> bool {
    if store.is_rate_limit_blocked(peer_ip) {
        return false;
    }
    if limiter.allow(peer_ip) {
        return true;
    }
    if let Some(peer_ip) = peer_ip {
        let _ = store.block_rate_limited_client_until(peer_ip, rate_limit_block_expires_at_ms());
    }
    false
}

fn rate_limit_block_expires_at_ms() -> u64 {
    unix_ms(
        SystemTime::now()
            .checked_add(CLUSTER_RATE_LIMIT_BLOCK_TTL)
            .unwrap_or_else(SystemTime::now),
    )
}

fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Returns the bench http.
pub fn bench_http(mut config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    config.bind_addr = "127.0.0.1:0".to_string();
    config.rate_limit_per_minute = 0;
    let requests = config.benchmark_requests;
    let concurrency = benchmark_concurrency(&config);
    let payload_bytes = config.benchmark_payload_bytes;
    let store = Arc::new(PublishStore::open(config)?);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server_store = Arc::clone(&store);
    thread::spawn(move || {
        let _ = run_listener(listener, server_store);
    });
    thread::sleep(Duration::from_millis(50));

    let payload = benchmark_payload(payload_bytes);
    let body = Arc::new(protocol::encode_publish_request_with_email(
        900,
        1,
        &payload,
        Some("bench@example.com"),
    ));
    let next = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut workers = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let body = Arc::clone(&body);
        let next = Arc::clone(&next);
        let failures = Arc::clone(&failures);
        workers.push(thread::spawn(move || loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= requests {
                break;
            }
            match post_binary(addr, &body, true) {
                Ok(response) if response.len() >= 14 && response[6] == 0 && response[7] == 0 => {}
                _ => {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for worker in workers {
        worker.join().map_err(|_| "benchmark worker panicked")?;
    }
    let failures = failures.load(Ordering::Relaxed);
    if failures != 0 {
        return Err(format!("{failures} benchmark requests failed").into());
    }
    let elapsed = start.elapsed();
    println!(
        "http_single_request_publish_rps={} requests={} concurrency={} live={}",
        (requests as f64 / elapsed.as_secs_f64()) as u64,
        requests,
        concurrency,
        store.stats().live
    );
    Ok(())
}

/// Returns the bench http receive.
pub fn bench_http_receive(mut config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    config.bind_addr = "127.0.0.1:0".to_string();
    config.rate_limit_per_minute = 0;
    let requests = config.benchmark_requests;
    let concurrency = benchmark_concurrency(&config);
    let payload_bytes = config.benchmark_payload_bytes;
    let store = Arc::new(PublishStore::open(config)?);

    let payload = benchmark_payload(payload_bytes);
    let publish_request =
        protocol::encode_publish_request_with_email(900, 1, &payload, Some("bench@example.com"));
    let decoded = protocol::decode_request(&publish_request, payload_bytes + 64)?;
    let mut codes = Vec::with_capacity(requests);
    for _ in 0..requests {
        let response = store.handle(decoded.operation, &decoded.payload);
        if response.len() < 14 || response[6] != 0 || response[7] != 0 {
            return Err("unable to preload publish for receive benchmark".into());
        }
        let mut reader = protocol::Reader::new(&response[14..]);
        reader.message_version()?;
        let code = reader.string()?;
        let _delete_token = reader.bytes()?;
        let _expires_at_ms = reader.u64()?;
        let _max_receives = reader.u16()?;
        let verification_url = reader.string()?;
        if let Some((verify_code, token)) = verification_query_parts(&verification_url) {
            if verify_code == code {
                let _ = store.verify_email(&verify_code, &token);
            }
        }
        codes.push(code);
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server_store = Arc::clone(&store);
    thread::spawn(move || {
        let _ = run_listener(listener, server_store);
    });
    thread::sleep(Duration::from_millis(50));

    let codes = Arc::new(codes);
    let next = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut workers = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let codes = Arc::clone(&codes);
        let next = Arc::clone(&next);
        let failures = Arc::clone(&failures);
        workers.push(thread::spawn(move || loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= codes.len() {
                break;
            }
            let body = protocol::encode_receive_request(&codes[index]);
            match post_binary(addr, &body, true) {
                Ok(response) if response.len() >= 14 && response[6] == 0 && response[7] == 0 => {}
                _ => {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for worker in workers {
        worker.join().map_err(|_| "benchmark worker panicked")?;
    }
    let failures = failures.load(Ordering::Relaxed);
    if failures != 0 {
        return Err(format!("{failures} receive benchmark requests failed").into());
    }
    let elapsed = start.elapsed();
    println!(
        "http_single_request_receive_rps={} requests={} concurrency={} live={}",
        (requests as f64 / elapsed.as_secs_f64()) as u64,
        requests,
        concurrency,
        store.stats().live
    );
    Ok(())
}

/// Returns the bench http flow.
pub fn bench_http_flow(mut config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    config.bind_addr = "127.0.0.1:0".to_string();
    config.rate_limit_per_minute = 0;
    let flows = config.benchmark_requests;
    let concurrency = benchmark_concurrency(&config);
    let payload_bytes = config.benchmark_payload_bytes;
    let preload_published_payloads = config.benchmark_preload_published_payloads;
    let store = Arc::new(PublishStore::open(config)?);

    let payload = benchmark_payload(payload_bytes);
    if preload_published_payloads > 0 {
        preload_live_published_payloads(&store, preload_published_payloads, &payload)?;
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server_store = Arc::clone(&store);
    thread::spawn(move || {
        let _ = run_listener(listener, server_store);
    });
    thread::sleep(Duration::from_millis(50));

    let publish_body = Arc::new(protocol::encode_publish_request_with_email(
        900,
        1,
        &payload,
        Some("bench@example.com"),
    ));
    let next = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut workers = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let publish_body = Arc::clone(&publish_body);
        let next = Arc::clone(&next);
        let failures = Arc::clone(&failures);
        workers.push(thread::spawn(move || loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= flows {
                break;
            }
            let publish_response = match post_binary(addr, &publish_body, true) {
                Ok(response) if response.len() >= 14 && response[6] == 0 && response[7] == 0 => {
                    response
                }
                _ => {
                    failures.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };
            let mut reader = protocol::Reader::new(&publish_response[14..]);
            if reader.message_version().is_err() {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            let publish_code = match reader.string() {
                Ok(code) => code,
                Err(_) => {
                    failures.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };
            if reader.bytes().is_err() || reader.u64().is_err() || reader.u16().is_err() {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            let verification_url = match reader.string() {
                Ok(url) => url,
                Err(_) => {
                    failures.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };
            let Some((verify_code, token)) = verification_query_parts(&verification_url) else {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            if verify_code != publish_code || get_verify(addr, &verify_code, &token).is_err() {
                failures.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            let receive_body = protocol::encode_receive_request(&publish_code);
            match post_binary(addr, &receive_body, true) {
                Ok(response) if response.len() >= 14 && response[6] == 0 && response[7] == 0 => {}
                _ => {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for worker in workers {
        worker.join().map_err(|_| "benchmark worker panicked")?;
    }
    let failures = failures.load(Ordering::Relaxed);
    if failures != 0 {
        return Err(format!("{failures} flow benchmark requests failed").into());
    }
    let elapsed = start.elapsed();
    let flow_rps = flows as f64 / elapsed.as_secs_f64();
    let total_rps = flow_rps * 2.0;
    println!(
        "http_single_request_flow_rps={} \
         http_single_request_total_rps={} \
         flows={} concurrency={} preloaded={} live={}",
        flow_rps as u64,
        total_rps as u64,
        flows,
        concurrency,
        preload_published_payloads,
        store.stats().live
    );
    Ok(())
}

fn preload_live_published_payloads(
    store: &PublishStore,
    count: usize,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let request =
        protocol::encode_publish_request_with_email(900, 1, payload, Some("bench@example.com"));
    let decoded = protocol::decode_request(&request, payload.len() + 64)?;
    for _ in 0..count {
        let response = store.handle(decoded.operation, &decoded.payload);
        if response.len() < 14 || response[6] != 0 || response[7] != 0 {
            return Err("unable to preload publish".into());
        }
        let mut reader = protocol::Reader::new(&response[14..]);
        reader.message_version()?;
        let code = reader.string()?;
        let _delete_token = reader.bytes()?;
        let _expires_at_ms = reader.u64()?;
        let _max_receives = reader.u16()?;
        let verification_url = reader.string()?;
        if let Some((verify_code, token)) = verification_query_parts(&verification_url) {
            if verify_code == code {
                let _ = store.verify_email(&verify_code, &token);
            }
        }
    }
    Ok(())
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

fn benchmark_concurrency(config: &ServerConfig) -> usize {
    if config.benchmark_concurrency == 0 {
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(4)
            .saturating_mul(16)
            .clamp(16, 128)
    } else {
        config.benchmark_concurrency
    }
}

fn post_binary(addr: SocketAddr, body: &[u8], close: bool) -> std::io::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(addr)?;
    post_binary_on_stream(&mut stream, addr, body, close)
}

fn get_verify(addr: SocketAddr, code: &str, token: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    let request = format!(
        "GET /v1/verify?code={code}&token={token} HTTP/1.1\r\n\
         Host: {addr}\r\n\
         Connection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    if response.starts_with(b"HTTP/1.1 200 ") {
        Ok(())
    } else {
        Err(std::io::Error::other("verification request failed"))
    }
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

fn post_binary_on_stream(
    stream: &mut TcpStream,
    addr: SocketAddr,
    body: &[u8],
    close: bool,
) -> std::io::Result<Vec<u8>> {
    let connection = if close { "close" } else { "keep-alive" };
    let header = format!(
        "POST /v1/publish HTTP/1.1\r\n\
         Host: {addr}\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Length: {}\r\n\
         Connection: {connection}\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        if let Some(pos) = find_header_end(&response) {
            break pos;
        }
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(Vec::new());
        }
        response.extend_from_slice(&chunk[..read]);
    };
    let headers = String::from_utf8_lossy(&response[..header_end]);
    let content_len = content_length(&headers).unwrap_or(0);
    let body_start = header_end + 4;
    while response.len() < body_start + content_len {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        response.extend_from_slice(&chunk[..read]);
    }
    Ok(response[body_start..body_start + content_len].to_vec())
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(headers: &str) -> Option<usize> {
    for line in headers.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("content-length") {
                return value.trim().parse().ok();
            }
        }
    }
    None
}

fn verify_error(title: &str, message: &str) -> crate::store::VerificationPage {
    crate::store::VerificationPage {
        success: false,
        title: title.to_string(),
        message: message.to_string(),
        email: None,
    }
}

fn render_verify_page(page: &crate::store::VerificationPage) -> String {
    let color = if page.success { "#146C2E" } else { "#B3261E" };
    let icon = if page.success {
        "check_circle"
    } else {
        "error"
    };
    let email = page
        .email
        .as_ref()
        .map(|email| {
            format!(
                "<p style=\"margin:16px 0 0;color:#49454F;font:500 14px Arial,sans-serif;\">{}</p>",
                escape_html(email)
            )
        })
        .unwrap_or_default();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>{}</title></head>\
<body style=\"margin:0;background:#FFFBFE;color:#1D1B20;font-family:Arial,sans-serif;\">\
<main style=\"min-height:100vh;display:flex;align-items:center;justify-content:center;padding:24px;box-sizing:border-box;\">\
<section style=\"max-width:520px;width:100%;border:1px solid #CAC4D0;border-radius:8px;padding:32px;background:#FFFBFE;box-sizing:border-box;\">\
<div style=\"width:48px;height:48px;border-radius:24px;background:{color};color:white;display:flex;align-items:center;justify-content:center;font:700 24px Arial,sans-serif;margin-bottom:20px;\">{icon}</div>\
<h1 style=\"margin:0 0 12px;font:500 28px Arial,sans-serif;color:#1D1B20;\">{}</h1>\
<p style=\"margin:0;color:#49454F;font:400 16px/1.5 Arial,sans-serif;\">{}</p>{email}\
</section></main></body></html>",
        escape_html(&page.title),
        escape_html(&page.title),
        escape_html(&page.message)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::RateLimiter;

    #[test]
    fn rate_limiter_enforces_burst_capacity() {
        let limiter = RateLimiter::new(60, 2);
        let ip = Some(IpAddr::V4(Ipv4Addr::LOCALHOST));

        assert!(limiter.allow(ip));
        assert!(limiter.allow(ip));
        assert!(!limiter.allow(ip));
    }

    #[test]
    fn zero_rate_limit_disables_limiter() {
        let limiter = RateLimiter::new(0, 1);
        let ip = Some(IpAddr::V4(Ipv4Addr::LOCALHOST));

        assert!(limiter.allow(ip));
        assert!(limiter.allow(ip));
        assert!(limiter.allow(None));
    }
}
