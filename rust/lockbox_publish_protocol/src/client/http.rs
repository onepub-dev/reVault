use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::client::{
    ClientError, Endpoint, HttpTransport, Scheme, Transport, DEFAULT_MAX_RESPONSE_BYTES,
};
use crate::protocol::ProtocolError;
use crate::topology::TopologyServer;

use super::helpers::parse_http_response;

const REQUEST_TOPOLOGY_HEADER: &str = "x-topology-version";
const SERVER_TOKEN_HEADER: &str = "x-lockbox-server-token";

impl HttpTransport {
    pub fn new(server_url: &str) -> Result<Self, ClientError> {
        Ok(Self {
            endpoint: Endpoint::parse(server_url)?,
            timeout: Duration::from_secs(10),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

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
            .map(|version| format!("{}: {version}\r\n", REQUEST_TOPOLOGY_HEADER))
            .unwrap_or_default();
        let server_token_header = server_token
            .map(|token| format!("{}: {token}\r\n", SERVER_TOKEN_HEADER))
            .unwrap_or_default();
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/octet-stream\r\n{topology_header}{server_token_header}Content-Length: {}\r\nConnection: close\r\n\r\n",
            self.endpoint.path,
            self.endpoint.host,
            body.len()
        );
        stream.write_all(request.as_bytes())?;
        stream.write_all(body)?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
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
        let (scheme, rest) = if let Some(rest) = server_url.strip_prefix("http://") {
            (Scheme::Http, rest)
        } else if let Some(rest) = server_url.strip_prefix("https://") {
            (Scheme::Https, rest)
        } else {
            return Err(ClientError::Url(
                "only http:// and https:// urls are supported".to_string(),
            ));
        };
        let (authority, path) = match rest.split_once('/') {
            Some((authority, path)) => (authority, format!("/{path}")),
            None => (rest, "/v1/publish".to_string()),
        };
        if authority.is_empty() {
            return Err(ClientError::Url("missing host".to_string()));
        }
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => {
                let port = port
                    .parse::<u16>()
                    .map_err(|_| ClientError::Url("invalid port".to_string()))?;
                (host.to_string(), port)
            }
            None => (
                authority.to_string(),
                match scheme {
                    Scheme::Http => 80,
                    Scheme::Https => 443,
                },
            ),
        };
        if host.is_empty() {
            return Err(ClientError::Url("missing host".to_string()));
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
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
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

fn tls_request(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    timeout: Duration,
    max_response_bytes: usize,
    topology_version: Option<u64>,
    server_token: Option<&str>,
) -> Result<Vec<u8>, ClientError> {
    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let request = match method {
        "GET" => agent
            .get(url)
            .set("Accept", "application/octet-stream")
            .set("Connection", "close"),
        "POST" => {
            let mut request = agent
                .post(url)
                .set("Content-Type", "application/octet-stream")
                .set("Accept", "application/octet-stream")
                .set("Connection", "close");
            if let Some(version) = topology_version {
                request = request.set(REQUEST_TOPOLOGY_HEADER, &version.to_string());
            }
            if let Some(token) = server_token {
                request = request.set(SERVER_TOKEN_HEADER, token);
            }
            request
        }
        other => return Err(ClientError::Http(format!("unsupported method {other}"))),
    };
    let response = match body {
        Some(body) => request.send_bytes(body),
        None => request.call(),
    }
    .map_err(ureq_error)?;
    if response.status() != 200 {
        return Err(ClientError::Http(format!(
            "HTTP/1.1 {} {}",
            response.status(),
            response.status_text()
        )));
    }
    read_ureq_body(response, max_response_bytes)
}

fn read_ureq_body(
    response: ureq::Response,
    max_response_bytes: usize,
) -> Result<Vec<u8>, ClientError> {
    let mut reader = response.into_reader().take(max_response_bytes as u64 + 1);
    let mut out = Vec::new();
    reader.read_to_end(&mut out)?;
    if out.len() > max_response_bytes {
        return Err(ClientError::Protocol(ProtocolError::PayloadTooLarge));
    }
    Ok(out)
}

fn ureq_error(err: ureq::Error) -> ClientError {
    match err {
        ureq::Error::Status(status, response) => {
            ClientError::Http(format!("HTTP/1.1 {status} {}", response.status_text()))
        }
        ureq::Error::Transport(transport) => ClientError::Http(transport.to_string()),
    }
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
