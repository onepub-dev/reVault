use super::{decode, encode, invalid, Request, Response, Result, MAX_MESSAGE_BYTES};
use std::time::Duration;

/// HTTPS relay client. Redirects are disabled to protect bearer capabilities.
pub struct ExchangeClient {
    origin: String,
    url: String,
    agent: ureq::Agent,
}

impl ExchangeClient {
    /// Connects to an explicit relay origin. Plain HTTP is allowed only on loopback.
    pub fn new(server: &str) -> Result<Self> {
        if server.len() > 2048 || server.chars().any(char::is_control) {
            return Err(invalid("invalid relay origin"));
        }
        let mut url = url::Url::parse(server).map_err(|_| invalid("invalid relay URL"))?;
        let loopback = url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if (url.scheme() != "https" && !(url.scheme() == "http" && loopback))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(invalid(
                "relay requires an HTTPS origin (HTTP only on loopback), without credentials/path/query/fragment",
            ));
        }
        let origin = url.origin().ascii_serialization();
        url.set_path("/v2/exchange");
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .new_agent();
        Ok(Self {
            origin,
            url: url.to_string(),
            agent,
        })
    }

    /// Returns the canonical origin pinned by the local invitation state.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Sends a bounded request and reports capability-free failures.
    pub fn request(&self, request: &Request) -> Result<Response> {
        let body = encode(request)?;
        let mut response = self
            .agent
            .post(&self.url)
            .header("content-type", "application/json")
            .send(&body)
            .map_err(|_| invalid("exchange relay request failed"))?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_MESSAGE_BYTES as u64)
            .read_to_vec()
            .map_err(|_| invalid("invalid relay response"))?;
        let response: Response = decode(&bytes)?;
        if let Some(error) = &response.error {
            return Err(super::ExchangeError(error.clone()));
        }
        Ok(response)
    }
}
