use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::client::{
    ClientError, Endpoint, HttpTransport, Scheme, Transport, DEFAULT_MAX_RESPONSE_BYTES,
};
use crate::protocol::ProtocolError;
use crate::topology::TopologyServer;
use url::Url;

use super::helpers::parse_http_response;

const REQUEST_TOPOLOGY_HEADER: &str = "x-topology-version";
const SERVER_TOKEN_HEADER: &str = "x-lockbox-server-token";
const MAX_HTTP_RESPONSE_HEADERS: usize = 64 * 1024;

impl HttpTransport {
    /// Creates a value from the supplied data.
    pub fn new(server_url: &str) -> Result<Self, ClientError> {
        Ok(Self {
            endpoint: Endpoint::parse(server_url)?,
            timeout: Duration::from_secs(10),
        })
    }

    /// Returns the with timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Returns get.
    pub fn get(url: &str, max_response_bytes: usize) -> Result<Vec<u8>, ClientError> {
        Endpoint::parse(url)?.get(Duration::from_secs(10), max_response_bytes)
    }

    fn post_binary_std(&self, body: &[u8]) -> Result<Vec<u8>, ClientError> {
        self.post_binary_std_with_headers(body, None, None)
    }

    fn post_binary_std_with_topology(
        &self,
        body: &[u8],
        topology_version: Option<u64>,
    ) -> Result<Vec<u8>, ClientError> {
        self.post_binary_std_with_headers(body, topology_version, None)
    }

    /// Returns the post binary with server token.
    pub fn post_binary_with_server_token(
        &self,
        body: &[u8],
        server_token: &str,
    ) -> Result<Vec<u8>, ClientError> {
        self.post_binary_std_with_headers(body, None, Some(server_token))
    }

    fn post_binary_std_with_headers(
        &self,
        body: &[u8],
        topology_version: Option<u64>,
        server_token: Option<&str>,
    ) -> Result<Vec<u8>, ClientError> {
        if self.endpoint.scheme == Scheme::Https {
            return tls_request(
                "POST",
                &self.endpoint.url(),
                Some(body),
                self.timeout,
                DEFAULT_MAX_RESPONSE_BYTES,
                topology_version,
                server_token,
            );
        }
        let mut stream = self.endpoint.connect(self.timeout)?;
        let topology_header = topology_version
            .map(|version| format!("{REQUEST_TOPOLOGY_HEADER}: {version}\r\n"))
            .unwrap_or_default();
        let server_token_header = server_token
            .map(|token| format!("{SERVER_TOKEN_HEADER}: {token}\r\n"))
            .unwrap_or_default();
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/octet-stream\r\n{topology_header}{server_token_header}Content-Length: {}\r\nConnection: close\r\n\r\n",
            self.endpoint.path,
            self.endpoint.host,
            body.len()
        );
        stream.write_all(request.as_bytes())?;
        stream.write_all(body)?;
        let response =
            read_plain_http_response(&mut stream, self.timeout, DEFAULT_MAX_RESPONSE_BYTES)?;
        parse_http_response(&response, DEFAULT_MAX_RESPONSE_BYTES)
    }
}

impl Transport for HttpTransport {
    fn post_binary(&self, body: &[u8]) -> Result<Vec<u8>, ClientError> {
        self.post_binary_std(body)
    }

    fn from_url(url: &str) -> Option<Self> {
        Self::new(url).ok()
    }

    fn get_topology(url: &str) -> Option<Vec<u8>> {
        let mut endpoint = Endpoint::parse(url).ok()?;
        endpoint.path = topology_path(&endpoint.path);
        endpoint
            .get(Duration::from_secs(10), DEFAULT_MAX_RESPONSE_BYTES)
            .ok()
    }

    fn post_binary_with_topology(
        &self,
        body: &[u8],
        topology_version: Option<u64>,
    ) -> Result<Vec<u8>, ClientError> {
        self.post_binary_std_with_topology(body, topology_version)
    }
}

impl Endpoint {
    pub(crate) fn parse(server_url: &str) -> Result<Self, ClientError> {
        let url = Url::parse(server_url).map_err(|err| ClientError::Url(err.to_string()))?;
        let scheme = match url.scheme() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            _ => {
                return Err(ClientError::Url(
                    "only http:// and https:// urls are supported".to_string(),
                ))
            }
        };
        let host = url
            .host_str()
            .filter(|host| !host.is_empty())
            .ok_or_else(|| ClientError::Url("missing host".to_string()))?
            .to_string();
        let port = url.port_or_known_default().ok_or_else(|| {
            ClientError::Url("missing port for unsupported URL scheme".to_string())
        })?;
        let has_explicit_path = server_url
            .split_once("://")
            .and_then(|(_, rest)| rest.find('/'))
            .is_some();
        let mut path = if has_explicit_path {
            url[url::Position::BeforePath..].to_string()
        } else {
            "/v1/publish".to_string()
        };
        if path.is_empty() {
            path = "/".to_string();
        }
        Ok(Self {
            scheme,
            host,
            port,
            path,
        })
    }

    fn connect(&self, timeout: Duration) -> Result<TcpStream, ClientError> {
        if self.scheme != Scheme::Http {
            return Err(ClientError::Url(
                "plain TCP connect is only valid for http:// endpoints".to_string(),
            ));
        }
        let stream = TcpStream::connect((&*self.host, self.port))?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(stream)
    }

    fn get(&self, timeout: Duration, max_response_bytes: usize) -> Result<Vec<u8>, ClientError> {
        if self.scheme == Scheme::Https {
            return tls_request(
                "GET",
                &self.url(),
                None,
                timeout,
                max_response_bytes,
                None,
                None,
            );
        }
        let mut stream = self.connect(timeout)?;
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/octet-stream\r\nConnection: close\r\n\r\n",
            self.path, self.host
        );
        stream.write_all(request.as_bytes())?;
        let response = read_plain_http_response(&mut stream, timeout, max_response_bytes)?;
        parse_http_response(&response, max_response_bytes)
    }

    pub(crate) fn url(&self) -> String {
        let scheme = match self.scheme {
            Scheme::Http => "http",
            Scheme::Https => "https",
        };
        let default_port = match self.scheme {
            Scheme::Http => 80,
            Scheme::Https => 443,
        };
        if self.port == default_port {
            format!("{scheme}://{}{}", self.host, self.path)
        } else {
            format!("{scheme}://{}:{}{}", self.host, self.port, self.path)
        }
    }
}

fn read_plain_http_response(
    stream: &mut TcpStream,
    timeout: Duration,
    max_body: usize,
) -> Result<Vec<u8>, ClientError> {
    let deadline = Instant::now() + timeout;
    let mut response = Vec::new();
    let mut body_start = None;
    let mut chunk = [0u8; 8192];
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "HTTP response exceeded the total read deadline",
            )));
        }
        stream.set_read_timeout(Some(remaining))?;
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(response);
        }
        response.extend_from_slice(&chunk[..count]);
        if body_start.is_none() {
            body_start = response
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|at| at + 4);
            if body_start.is_none() && response.len() > MAX_HTTP_RESPONSE_HEADERS {
                return Err(ClientError::Http(
                    "response headers are too large".to_string(),
                ));
            }
        }
        if let Some(start) = body_start {
            if start > MAX_HTTP_RESPONSE_HEADERS {
                return Err(ClientError::Http(
                    "response headers are too large".to_string(),
                ));
            }
            if response.len() - start > max_body {
                return Err(ClientError::Protocol(ProtocolError::PayloadTooLarge));
            }
        }
    }
}

fn tls_request(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    timeout: Duration,
    max_response_bytes: usize,
    topology_version: Option<u64>,
    server_token: Option<&str>,
) -> Result<Vec<u8>, ClientError> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .new_agent();
    let response = match method {
        "GET" => agent
            .get(url)
            .header("Accept", "application/octet-stream")
            .header("Connection", "close")
            .call(),
        "POST" => {
            let mut request = agent
                .post(url)
                .header("Content-Type", "application/octet-stream")
                .header("Accept", "application/octet-stream")
                .header("Connection", "close");
            if let Some(version) = topology_version {
                request = request.header(REQUEST_TOPOLOGY_HEADER, version.to_string());
            }
            if let Some(token) = server_token {
                request = request.header(SERVER_TOKEN_HEADER, token);
            }
            request.send(body.unwrap_or_default())
        }
        other => return Err(ClientError::Http(format!("unsupported method {other}"))),
    }
    .map_err(ureq_error)?;
    if response.status().as_u16() != 200 {
        return Err(ClientError::Http(format!(
            "HTTP/1.1 {}",
            response.status().as_u16(),
        )));
    }
    read_ureq_body(response, max_response_bytes)
}

fn read_ureq_body(
    mut response: ureq::http::Response<ureq::Body>,
    max_response_bytes: usize,
) -> Result<Vec<u8>, ClientError> {
    let out = response
        .body_mut()
        .with_config()
        .limit(max_response_bytes as u64 + 1)
        .read_to_vec()
        .map_err(|err| ClientError::Http(err.to_string()))?;
    if out.len() > max_response_bytes {
        return Err(ClientError::Protocol(ProtocolError::PayloadTooLarge));
    }
    Ok(out)
}

fn ureq_error(err: ureq::Error) -> ClientError {
    ClientError::Http(err.to_string())
}

pub(crate) fn topology_urls_from_servers(servers: &[TopologyServer]) -> Vec<String> {
    dedupe_urls(
        servers
            .iter()
            .filter_map(|server| topology_url_from_publish_url(&server.url)),
    )
}

fn topology_path(path: &str) -> String {
    let _ = path;
    "/v1/topology".to_string()
}

pub(crate) fn topology_url_from_publish_url(url: &str) -> Option<String> {
    let mut endpoint = Endpoint::parse(url).ok()?;
    endpoint.path = topology_path(&endpoint.path);
    Some(endpoint.url())
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

#[cfg(test)]
mod tests {
    use super::{topology_path, topology_url_from_publish_url};

    fn response_fixture(response: Vec<u8>, limit: usize) -> Result<Vec<u8>, super::ClientError> {
        use std::io::Write;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let writer = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let _ = socket.write_all(&response);
        });
        let mut socket = std::net::TcpStream::connect(address).unwrap();
        let result =
            super::read_plain_http_response(&mut socket, std::time::Duration::from_secs(2), limit);
        drop(socket);
        writer.join().unwrap();
        result
    }

    #[test]
    fn plain_http_rejects_oversized_headers_and_body_before_eof() {
        let oversized_headers = vec![b'X'; super::MAX_HTTP_RESPONSE_HEADERS + 8192];
        assert!(matches!(
            response_fixture(oversized_headers, 32),
            Err(super::ClientError::Http(_))
        ));
        let mut oversized_body = b"HTTP/1.1 200 OK\r\nContent-Length: 65536\r\n\r\n".to_vec();
        oversized_body.extend_from_slice(&[b'B'; 8192]);
        assert!(matches!(
            response_fixture(oversized_body, 32),
            Err(super::ClientError::Protocol(
                crate::protocol::ProtocolError::PayloadTooLarge
            ))
        ));
        let valid = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc";
        assert_eq!(response_fixture(valid.to_vec(), 3).unwrap(), valid);
    }

    #[test]
    fn plain_http_slow_drip_cannot_extend_total_read_deadline() {
        use std::io::Write;
        use std::time::{Duration, Instant};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let writer = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            if socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 40\r\n\r\n")
                .is_err()
            {
                return;
            }
            for _ in 0..40 {
                if socket.write_all(b"x").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        let mut socket = std::net::TcpStream::connect(address).unwrap();
        let started = Instant::now();
        let result = super::read_plain_http_response(&mut socket, Duration::from_millis(400), 64);
        let elapsed = started.elapsed();
        let _ = socket.shutdown(std::net::Shutdown::Both);
        drop(socket);
        writer.join().unwrap();
        assert!(matches!(result, Err(super::ClientError::Io(error))
            if matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock)));
        assert!(
            elapsed < Duration::from_secs(2),
            "slow drip extended the deadline: {elapsed:?}"
        );
    }

    #[test]
    fn topology_path_strips_existing_query_token() {
        assert_eq!(
            topology_path("/v1/topology?token=server-token"),
            "/v1/topology"
        );
        assert_eq!(
            topology_path("/v1/publish?token=server-token"),
            "/v1/topology"
        );
        assert_eq!(topology_path("/v1/publish"), "/v1/topology");
        assert_eq!(
            topology_url_from_publish_url("https://server.example/v1/publish?token=server-token")
                .as_deref(),
            Some("https://server.example/v1/topology")
        );
    }
}
