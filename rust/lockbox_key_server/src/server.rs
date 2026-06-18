use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::server_log::log_server_event;
use crate::store::{PublishStore, ServerConfig};
use lockbox_publish_protocol::payload;
use lockbox_publish_protocol::protocol::{self, Operation, Status};
use lockbox_publish_protocol::status;
use lockbox_publish_protocol::topology;

const MAX_HTTP_HEADER: usize = 16 * 1024;
const MAX_WIRE_OVERHEAD: usize = 128;
const REQUEST_IO_TIMEOUT: Duration = Duration::from_secs(10);
const CLUSTER_RATE_LIMIT_BLOCK_TTL: Duration = Duration::from_secs(24 * 60 * 60);

pub fn run_server(bind: &str, store: Arc<PublishStore>) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    run_listener(listener, store)
}

pub fn run_listener(listener: TcpListener, store: Arc<PublishStore>) -> std::io::Result<()> {
    let local_addr = listener.local_addr()?;
    log_server_event(format!("lockbox_key_server listening on {local_addr}"));
    let purge_store = Arc::clone(&store);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        purge_store.purge_expired();
    });
    let compact_store = Arc::clone(&store);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));
        if let Err(err) = compact_store.compact_if_needed() {
            log_server_event(format!("compaction failed: {err}"));
        }
    });

    let worker_count = worker_count();
    let (tx, rx) = mpsc::sync_channel::<TcpStream>(accepted_stream_queue_bound(worker_count));
    let rx = Arc::new(Mutex::new(rx));
    let limiter = Arc::new(RateLimiter::new(
        store.rate_limit_per_minute(),
        store.rate_limit_burst(),
    ));
    for worker_id in 0..worker_count {
        let store = Arc::clone(&store);
        let rx = Arc::clone(&rx);
        let limiter = Arc::clone(&limiter);
        thread::Builder::new()
            .name(format!("publish-http-{worker_id}"))
            .stack_size(256 * 1024)
            .spawn(move || loop {
                let stream = {
                    let Ok(guard) = rx.lock() else {
                        break;
                    };
                    guard.recv()
                };
                match stream {
                    Ok(stream) => {
                        let _ = handle_stream(stream, Arc::clone(&store), Arc::clone(&limiter));
                    }
                    Err(_) => break,
                }
            })?;
    }

    let mut last_accept_error_log = Instant::now() - Duration::from_secs(30);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if tx.send(stream).is_err() {
                    break;
                }
            }
            Err(err) => {
                if is_accept_resource_pressure(&err) {
                    if last_accept_error_log.elapsed() >= Duration::from_secs(10) {
                        log_server_event(format!("accept deferred under resource pressure: {err}"));
                        last_accept_error_log = Instant::now();
                    }
                    thread::sleep(Duration::from_millis(50));
                } else {
                    log_server_event(format!("accept failed: {err}"));
                }
            }
        }
    }
    Ok(())
}

pub fn local_addr(listener: &TcpListener) -> std::io::Result<SocketAddr> {
    listener.local_addr()
}

fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .saturating_mul(4)
        .clamp(4, 64)
}

fn accepted_stream_queue_bound(worker_count: usize) -> usize {
    worker_count.saturating_mul(4).clamp(16, 256)
}

fn is_accept_resource_pressure(err: &std::io::Error) -> bool {
    matches!(err.kind(), ErrorKind::WouldBlock) || matches!(err.raw_os_error(), Some(11 | 23 | 24))
}

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

pub fn handle_stream(
    mut stream: TcpStream,
    store: Arc<PublishStore>,
    limiter: Arc<RateLimiter>,
) -> std::io::Result<()> {
    configure_stream_deadlines(&stream)?;
    let mut buffer = Vec::with_capacity(MAX_HTTP_HEADER);
    let mut chunk = [0_u8; 1024];
    let peer_ip = stream.peer_addr().ok().map(|addr| addr.ip());
    loop {
        let header_end = loop {
            if let Some(pos) = find_header_end(&buffer) {
                break pos;
            }
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..read]);
            if buffer.len() > MAX_HTTP_HEADER + store.max_payload_bytes() + MAX_WIRE_OVERHEAD {
                write_response(
                    &mut stream,
                    protocol::encode_error(
                        Operation::Publish,
                        Status::PayloadTooLarge,
                        "too large",
                    ),
                    true,
                )?;
                return Ok(());
            }
        };
        let headers = String::from_utf8_lossy(&buffer[..header_end]);
        let close = wants_close(&headers);
        let mut lines = headers.lines();
        let request_line = lines.next().unwrap_or_default();
        let server_token_authenticated = request_has_server_token(&headers, &store);
        if request_line.starts_with("GET /v1/verify") {
            if !server_token_authenticated && !allow_anonymous_request(&store, &limiter, peer_ip) {
                write_response(
                    &mut stream,
                    protocol::encode_error(Operation::Publish, Status::RateLimited, "rate limited"),
                    true,
                )?;
                return Ok(());
            }
            let page = handle_verify_request(request_line, &store);
            write_html(
                &mut stream,
                if page.success { 200 } else { 400 },
                &render_verify_page(&page),
            )?;
            return Ok(());
        }
        if topology_request_target(request_line).is_some() {
            if !server_token_authenticated && !allow_anonymous_request(&store, &limiter, peer_ip) {
                write_response(
                    &mut stream,
                    protocol::encode_error(Operation::Publish, Status::RateLimited, "rate limited"),
                    true,
                )?;
                return Ok(());
            }
            match topology::encode_topology(&store.topology()) {
                Ok(body) => write_binary(&mut stream, 200, &body)?,
                Err(err) => write_plain(&mut stream, 500, err.to_string().as_bytes())?,
            }
            return Ok(());
        }
        if status_request_target(request_line).is_some() {
            if !server_token_authenticated && !allow_anonymous_request(&store, &limiter, peer_ip) {
                write_response(
                    &mut stream,
                    protocol::encode_error(Operation::Publish, Status::RateLimited, "rate limited"),
                    true,
                )?;
                return Ok(());
            }
            let body = status::encode_status(&store.status_document());
            write_binary(&mut stream, 200, &body)?;
            return Ok(());
        }
        let topology_registration_endpoint =
            request_line.starts_with("POST /v1/topology/register ");
        let replicate_endpoint = if request_line.starts_with("POST /v1/publish ") {
            false
        } else if request_line.starts_with("POST /v1/replicate ") {
            true
        } else if topology_registration_endpoint {
            false
        } else {
            write_plain(&mut stream, 404, b"not found")?;
            return Ok(());
        };
        let inter_server_endpoint_authenticated =
            (topology_registration_endpoint || replicate_endpoint) && server_token_authenticated;
        if !inter_server_endpoint_authenticated
            && !allow_anonymous_request(&store, &limiter, peer_ip)
        {
            write_response(
                &mut stream,
                protocol::encode_error(Operation::Publish, Status::RateLimited, "rate limited"),
                true,
            )?;
            return Ok(());
        }
        let content_len = content_length(&headers).unwrap_or(0);
        if content_len > store.max_payload_bytes() + MAX_WIRE_OVERHEAD {
            write_response(
                &mut stream,
                protocol::encode_error(Operation::Publish, Status::PayloadTooLarge, "too large"),
                true,
            )?;
            return Ok(());
        }
        let body_start = header_end + 4;
        while buffer.len() < body_start + content_len {
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..read]);
        }
        let body_end = body_start + content_len;
        let body = buffer[body_start..body_end].to_vec();
        let response = if topology_registration_endpoint {
            match store.handle_topology_registration(&body) {
                Ok(response) => response,
                Err(err) => protocol::encode_error(
                    Operation::Publish,
                    Status::StoreUnavailable,
                    &err.to_string(),
                ),
            }
        } else {
            match protocol::decode_request(&body, store.max_payload_bytes() + MAX_WIRE_OVERHEAD) {
                Ok(request) => {
                    let _ = request.flags;
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
        };
        write_response(&mut stream, response, close)?;
        buffer.drain(..body_end);
        if close {
            return Ok(());
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

fn configure_stream_deadlines(stream: &TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(REQUEST_IO_TIMEOUT))?;
    stream.set_write_timeout(Some(REQUEST_IO_TIMEOUT))
}

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

fn write_response(stream: &mut TcpStream, body: Vec<u8>, close: bool) -> std::io::Result<()> {
    let connection = if close { "close" } else { "keep-alive" };
    let header = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Length: {}\r\n\
         Connection: {connection}\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}

fn write_plain(stream: &mut TcpStream, status: u16, body: &[u8]) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {status} Error\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn write_html(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let reason = if status == 200 { "OK" } else { "Error" };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    Ok(())
}

fn write_binary(stream: &mut TcpStream, status: u16, body: &[u8]) -> std::io::Result<()> {
    let reason = if status == 200 { "OK" } else { "Error" };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
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

fn topology_request_target(request_line: &str) -> Option<&str> {
    let mut parts = request_line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let target = parts.next()?;
    if target.split_once('?').map_or(target, |(path, _)| path) == "/v1/topology" {
        Some(target)
    } else {
        None
    }
}

fn status_request_target(request_line: &str) -> Option<&str> {
    let mut parts = request_line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let target = parts.next()?;
    if target.split_once('?').map_or(target, |(path, _)| path) == "/v1/status" {
        Some(target)
    } else {
        None
    }
}

fn request_has_server_token(headers: &str, store: &PublishStore) -> bool {
    header_value(headers, "authorization")
        .and_then(bearer_token)
        .or_else(|| header_value(headers, "x-lockbox-server-token"))
        .or_else(|| header_value(headers, "x-topology-token"))
        .as_deref()
        .is_some_and(|token| store.topology_token_matches(token))
}

fn bearer_token(value: String) -> Option<String> {
    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    for line in headers.lines().skip(1) {
        let (key, value) = line.split_once(':')?;
        if key.eq_ignore_ascii_case(name) {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn handle_verify_request(
    request_line: &str,
    store: &PublishStore,
) -> crate::store::VerificationPage {
    let Some(target) = request_line.split_whitespace().nth(1) else {
        return verify_error(
            "Verification failed",
            "The verification request is malformed.",
        );
    };
    let Some((_, query)) = target.split_once('?') else {
        return verify_error(
            "Verification failed",
            "The verification link is missing its token.",
        );
    };
    let code = query_param(query, "code");
    let token = query_param(query, "token");
    match (code, token) {
        (Some(code), Some(token)) => store.verify_email(&code, &token),
        _ => verify_error(
            "Verification failed",
            "The verification link is missing its token.",
        ),
    }
}

fn verify_error(title: &str, message: &str) -> crate::store::VerificationPage {
    crate::store::VerificationPage {
        success: false,
        title: title.to_string(),
        message: message.to_string(),
        email: None,
    }
}

fn query_param(query: &str, name: &str) -> Option<String> {
    for part in query.split('&') {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        if key == name {
            return Some(percent_decode(value));
        }
    }
    None
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_digit(bytes[index + 1]), hex_digit(bytes[index + 2]))
            {
                out.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        out.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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

fn wants_close(headers: &str) -> bool {
    headers.lines().any(|line| {
        line.split_once(':')
            .map(|(key, value)| {
                key.eq_ignore_ascii_case("connection") && value.trim().eq_ignore_ascii_case("close")
            })
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read, Write};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::thread;

    use super::{configure_stream_deadlines, unix_ms, RateLimiter, REQUEST_IO_TIMEOUT};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use crate::store::{PublishStore, ServerConfig};
    use lockbox_publish_protocol::{
        decode_status, decode_topology, encode_replication_request, encode_topology_registration,
        protocol, sign_replication_event, ReplicationEvent, ReplicationEventKind,
        ReplicationRequest, ServerStatus, TopologyRegistration,
    };

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

    #[test]
    fn topology_request_target_matches_only_topology_gets() {
        assert_eq!(
            super::topology_request_target("GET /v1/topology HTTP/1.1"),
            Some("/v1/topology")
        );
        assert_eq!(
            super::topology_request_target("GET /v1/topology?token=abc HTTP/1.1"),
            Some("/v1/topology?token=abc")
        );
        assert_eq!(
            super::topology_request_target("GET /v1/topologyx HTTP/1.1"),
            None
        );
        assert_eq!(
            super::topology_request_target("POST /v1/topology HTTP/1.1"),
            None
        );
    }

    #[test]
    fn topology_get_rate_limits_clients_but_allows_token_requests() {
        let (_guard, mut config) = temp_server_config("topology-rate-limit");
        config.topology_token = Some("server-token".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));

        let first = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/topology");
        decode_topology(http_body(&first)).unwrap();

        let second = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/topology");
        let response = protocol::decode_response(http_body(&second), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let query_token = topology_get(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/topology?token=server-token",
        );
        let response = protocol::decode_response(http_body(&query_token), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let authenticated = topology_get_with_headers(
            store,
            limiter,
            "/v1/topology",
            "X-Lockbox-Server-Token: server-token\r\n",
        );
        decode_topology(http_body(&authenticated)).unwrap();
    }

    #[test]
    fn status_rate_limits_clients_but_allows_token_requests() {
        let (_guard, mut config) = temp_server_config("status-rate-limit");
        config.topology_token = Some("server-token".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));

        let first = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/status");
        decode_status(http_body(&first)).unwrap();

        let second = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/status");
        let response = protocol::decode_response(http_body(&second), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let authenticated = topology_get_with_headers(
            store,
            limiter,
            "/v1/status",
            "Authorization: Bearer server-token\r\n",
        );
        decode_status(http_body(&authenticated)).unwrap();
    }

    #[test]
    fn rate_limit_violation_blocks_client_across_refilled_buckets() {
        let (_guard, mut config) = temp_server_config("cluster-rate-limit-block");
        config.topology_token = Some("server-token".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));

        let first = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/topology");
        decode_topology(http_body(&first)).unwrap();

        let second = topology_get(Arc::clone(&store), Arc::clone(&limiter), "/v1/topology");
        let response = protocol::decode_response(http_body(&second), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let refilled_limiter = Arc::new(RateLimiter::new(60, 1));
        let blocked = topology_get(Arc::clone(&store), refilled_limiter, "/v1/topology");
        let response = protocol::decode_response(http_body(&blocked), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let authenticated = topology_get_with_headers(
            store,
            Arc::new(RateLimiter::new(60, 1)),
            "/v1/topology",
            "X-Lockbox-Server-Token: server-token\r\n",
        );
        decode_topology(http_body(&authenticated)).unwrap();
    }

    #[test]
    fn topology_registration_rate_limits_without_header_but_allows_token_header() {
        let (_guard, mut config) = temp_server_config("topology-registration-rate-limit");
        config.topology_token = Some("server-token".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));
        let body = encode_topology_registration(&TopologyRegistration {
            cluster_id: "default".to_string(),
            server_id: 1,
            server_url: "http://peer.example/v1/publish".to_string(),
            status: ServerStatus::Active,
            security_token: "server-token".to_string(),
        })
        .unwrap();

        let first = post_request(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/topology/register",
            "",
            &body,
        );
        decode_topology(http_body(&first)).unwrap();

        let second = post_request(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/topology/register",
            "",
            &body,
        );
        let response = protocol::decode_response(http_body(&second), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let authenticated = post_request(
            store,
            limiter,
            "/v1/topology/register",
            "X-Lockbox-Server-Token: server-token\r\n",
            &body,
        );
        decode_topology(http_body(&authenticated)).unwrap();
    }

    #[test]
    fn replication_rate_limits_anonymous_but_allows_token_requests() {
        let (_guard, mut config) = temp_server_config("replication-rate-limit");
        config.server_id = 1;
        config.topology_token = Some("server-token".to_string());
        config.replication_token = Some("peer-secret".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));
        let first_body = rate_limit_block_request(1, [203, 0, 113, 21]);
        let second_body = rate_limit_block_request(2, [203, 0, 113, 22]);

        let first = post_request(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/replicate",
            "",
            &first_body,
        );
        let response = protocol::decode_response(http_body(&first), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::Success);

        let anonymous = post_request(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/replicate",
            "",
            &second_body,
        );
        let response = protocol::decode_response(http_body(&anonymous), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);

        let authenticated = post_request(
            Arc::clone(&store),
            limiter,
            "/v1/replicate",
            "X-Lockbox-Server-Token: server-token\r\n",
            &second_body,
        );
        let response = protocol::decode_response(http_body(&authenticated), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::Success);
        assert!(store.is_rate_limit_blocked(Some(IpAddr::from([203, 0, 113, 22]))));
    }

    #[test]
    fn server_token_header_does_not_bypass_publish_rate_limit() {
        let (_guard, mut config) = temp_server_config("publish-token-rate-limit");
        config.topology_token = Some("server-token".to_string());
        let store = Arc::new(PublishStore::open(config).unwrap());
        let limiter = Arc::new(RateLimiter::new(60, 1));
        let body = b"not-a-protocol-request";

        let first = post_request(
            Arc::clone(&store),
            Arc::clone(&limiter),
            "/v1/publish",
            "X-Lockbox-Server-Token: server-token\r\n",
            body,
        );
        let response = protocol::decode_response(http_body(&first), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::MalformedRequest);

        let second = post_request(
            store,
            limiter,
            "/v1/publish",
            "X-Lockbox-Server-Token: server-token\r\n",
            body,
        );
        let response = protocol::decode_response(http_body(&second), 1024).unwrap();
        assert_eq!(response.status, protocol::Status::RateLimited);
    }

    #[test]
    fn stream_deadlines_are_configured_for_requests() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == ErrorKind::PermissionDenied => return,
            Err(err) => panic!("unable to bind local test listener: {err}"),
        };
        let addr = listener.local_addr().unwrap();
        let client = thread::spawn(move || TcpStream::connect(addr).unwrap());
        let (server, _) = listener.accept().unwrap();
        let client = client.join().unwrap();

        configure_stream_deadlines(&server).unwrap();

        assert_eq!(server.read_timeout().unwrap(), Some(REQUEST_IO_TIMEOUT));
        assert_eq!(server.write_timeout().unwrap(), Some(REQUEST_IO_TIMEOUT));
        drop(client);
    }

    fn topology_get(store: Arc<PublishStore>, limiter: Arc<RateLimiter>, target: &str) -> Vec<u8> {
        topology_get_with_headers(store, limiter, target, "")
    }

    fn topology_get_with_headers(
        store: Arc<PublishStore>,
        limiter: Arc<RateLimiter>,
        target: &str,
        headers: &str,
    ) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            super::handle_stream(stream, store, limiter).unwrap();
        });
        let response = get_request(addr, target, headers);
        server.join().unwrap();
        response
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

    fn post_request(
        store: Arc<PublishStore>,
        limiter: Arc<RateLimiter>,
        target: &str,
        headers: &str,
        body: &[u8],
    ) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            super::handle_stream(stream, store, limiter).unwrap();
        });
        let mut stream = TcpStream::connect(addr).unwrap();
        let request = format!(
            "POST {target} HTTP/1.1\r\nHost: {addr}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let mut wire = request.into_bytes();
        wire.extend_from_slice(body);
        let _ = stream.write_all(&wire);
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        server.join().unwrap();
        response
    }

    fn rate_limit_block_request(sequence: u64, ip: [u8; 4]) -> Vec<u8> {
        let event = ReplicationEvent {
            origin_server_id: 0,
            origin_epoch: 2,
            origin_sequence: sequence,
            kind: ReplicationEventKind::RateLimitBlock {
                client_ip: IpAddr::from(ip).to_string(),
                expires_at_unix_ms: unix_ms(SystemTime::now() + Duration::from_secs(60)),
            },
        };
        encode_replication_request(&ReplicationRequest {
            authentication: sign_replication_event(b"peer-secret", &event),
            event,
        })
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

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "lockbox-key-server-{label}-{}-{}",
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
}
