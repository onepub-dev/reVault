//! Native HTTPS with HTTP-01 validation and persistent certificate renewal.

use std::io;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use futures_util::StreamExt;
use rustls_acme::{caches::DirCache, AcmeConfig, UseChallenge};

use crate::server::{make_app, start_background_maintenance};
use crate::server_log::log_server_event;
use crate::store::PublishStore;

/// Let's Encrypt settings. Production issuance requires explicit opt-in.
#[derive(Clone, Debug)]
pub struct AcmeSettings {
    /// DNS hostname to certify. Empty disables native HTTPS.
    pub domain: String,
    /// ACME account contact email.
    pub email: String,
    /// Private persistent root for account keys and certificates.
    pub cache_dir: PathBuf,
    /// Use the production CA instead of the staging CA.
    pub production: bool,
    /// HTTP-01 listener port on the same IP as HTTPS (normally 80).
    pub http_port: u16,
}

impl Default for AcmeSettings {
    fn default() -> Self {
        Self {
            domain: String::new(),
            email: String::new(),
            cache_dir: PathBuf::from("/var/lib/revault-key-server/acme"),
            production: false,
            http_port: 80,
        }
    }
}

impl AcmeSettings {
    /// Validate configuration before opening state or contacting a CA.
    pub fn validate(&self, bind: &str) -> io::Result<SocketAddr> {
        let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
        let addr: SocketAddr = bind
            .parse()
            .map_err(|_| invalid("HTTPS bind must be an IP address and port"))?;
        if addr.ip().is_unspecified() || addr.ip().is_multicast() {
            return Err(invalid("HTTPS requires a specific unicast bind address"));
        }
        if addr.port() == 0 || self.http_port == 0 || addr.port() == self.http_port {
            return Err(invalid("HTTPS and HTTP-01 require distinct nonzero ports"));
        }
        if self.domain.len() > 253
            || !self.domain.contains('.')
            || self.domain.parse::<std::net::IpAddr>().is_ok()
            || self.domain.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-')
            })
        {
            return Err(invalid(
                "acme_domain must be a DNS hostname without scheme, port or wildcard",
            ));
        }
        if !self.email.contains('@')
            || self
                .email
                .bytes()
                .any(|c| c.is_ascii_whitespace() || c.is_ascii_control())
        {
            return Err(invalid("acme_email must be a contact email address"));
        }
        if !self.cache_dir.is_absolute() {
            return Err(invalid(
                "acme_cache_dir must be an absolute persistent path",
            ));
        }
        Ok(addr)
    }

    fn cache_path(&self) -> PathBuf {
        self.cache_dir.join(if self.production {
            "production"
        } else {
            "staging"
        })
    }
}

fn private_cache(path: &std::path::Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
        if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ACME cache must not be a symlink",
            ));
        }
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    std::fs::create_dir_all(path)?;
    Ok(())
}

fn challenge_app(service: rustls_acme::tower::TowerHttp01ChallengeService) -> Router {
    // No key-server API is exposed over plaintext HTTP.
    Router::new().route_service("/.well-known/acme-challenge/{token}", service)
}

/// Serve the key API over HTTPS and renew certificates without restarting.
/// Both listeners use the configured IP; HTTP serves only ACME challenges.
pub fn run_https(bind: &str, settings: AcmeSettings, store: Arc<PublishStore>) -> io::Result<()> {
    let https_addr = settings.validate(bind)?;
    let http_addr = SocketAddr::new(https_addr.ip(), settings.http_port);
    // Bind both sockets before starting issuance or background workers.
    let https_listener = TcpListener::bind(https_addr)?;
    let http_listener = TcpListener::bind(http_addr)?;
    https_listener.set_nonblocking(true)?;
    http_listener.set_nonblocking(true)?;
    let cache_path = settings.cache_path();
    private_cache(&cache_path)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let mut state = AcmeConfig::new([settings.domain])
            .contact([format!("mailto:{}", settings.email)])
            .cache(DirCache::new(cache_path))
            .directory_lets_encrypt(settings.production)
            .challenge_type(UseChallenge::Http01)
            .state();
        let acceptor = state.axum_acceptor(state.default_rustls_config());
        let challenges = challenge_app(state.http01_challenge_tower_service());
        let renewal = async move {
            while let Some(event) = state.next().await {
                match event {
                    Ok(event) => log_server_event(format!("ACME certificate event: {event:?}")),
                    Err(err) => log_server_event(format!(
                        "ACME certificate acquisition/renewal failed; retrying: {err}"
                    )),
                }
            }
            Err::<(), io::Error>(io::Error::other("ACME renewal worker stopped"))
        };
        start_background_maintenance(&store);
        log_server_event(format!(
            "HTTPS listening on {https_addr}; ACME HTTP-01 on {http_addr}"
        ));
        tokio::try_join!(
            axum_server::from_tcp(https_listener)?
                .acceptor(acceptor)
                .serve(make_app(store).into_make_service_with_connect_info::<SocketAddr>()),
            axum_server::from_tcp(http_listener)?.serve(challenges.into_make_service()),
            renewal,
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> AcmeSettings {
        AcmeSettings {
            domain: "revaultkeyshare0.onepub.dev".into(),
            email: "ops@example.com".into(),
            ..Default::default()
        }
    }

    #[test]
    fn dedicated_ipv6_and_separate_staging_cache() {
        let mut config = settings();
        assert!(config.validate("[2600:1900:4180:bfa7:0:1:0:2]:443").is_ok());
        let staging = config.cache_path();
        config.production = true;
        assert_ne!(config.cache_path(), staging);
    }

    #[test]
    fn refuses_wildcards_conflicting_ports_and_invalid_names() {
        let mut config = settings();
        for addr in ["[::]:443", "0.0.0.0:443", "[::1]:80", "[::1]:0"] {
            assert!(config.validate(addr).is_err(), "{addr}");
        }
        for domain in [
            "*.example.com",
            "https://example.com",
            "example.com:443",
            "a..com",
            "127.0.0.1",
        ] {
            config.domain = domain.into();
            assert!(config.validate("[::1]:443").is_err(), "{domain}");
        }
    }

    #[test]
    fn cache_directory_is_private() {
        let path = std::env::temp_dir().join(format!("revault-acme-cache-{}", std::process::id()));
        private_cache(&path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        std::fs::remove_dir(path).unwrap();
    }

    #[tokio::test]
    async fn plaintext_listener_does_not_expose_key_api() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;
        let state = AcmeConfig::new(["keyshare0.example.com"])
            .cache(DirCache::new("/unused-cache"))
            .challenge_type(UseChallenge::Http01)
            .state();
        let app = challenge_app(state.http01_challenge_tower_service());
        for path in [
            "/v1/status",
            "/v1/publish",
            "/v2/exchange",
            "/.well-known/acme-challenge/unknown",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        }
    }
}
