//! Exercise the public server with a cached test certificate, without a live CA.
use std::io;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use rustls_acme::futures_rustls::rustls;
use rustls_acme::{caches::DirCache, CertCache};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn native_https_validates_certificate_and_reuses_it_after_restart() {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("acme/staging");
    std::fs::create_dir_all(&cache).unwrap();
    let cert = rcgen::generate_simple_self_signed(vec!["keyshare.test".into()]).unwrap();
    let cached = format!("{}\n{}", cert.key_pair.serialize_pem(), cert.cert.pem());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime
        .block_on(DirCache::new(cache).store_cert(
            &["keyshare.test".into()],
            "https://acme-staging-v02.api.letsencrypt.org/directory",
            cached.as_bytes(),
        ))
        .unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.cert.der().clone()).unwrap();
    let client = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client));
    for _ in 0..2 {
        // Reserve separate ports, then release them immediately before startup.
        let https = TcpListener::bind("[::1]:0").unwrap();
        let http = TcpListener::bind("[::1]:0").unwrap();
        let addr = https.local_addr().unwrap();
        let config = directory.path().join("server.toml");
        std::fs::write(&config, format!(
            "bind_addr = \"{addr}\"\nstate_dir = {:?}\nacme_domain = \"keyshare.test\"\nacme_email = \"ops@example.com\"\nacme_cache_dir = {:?}\nacme_http_port = {}\n",
            directory.path().join("state").to_str().unwrap(),
            directory.path().join("acme").to_str().unwrap(),
            http.local_addr().unwrap().port(),
        )).unwrap();
        drop((https, http));
        let errors = directory.path().join("startup.stderr");
        let mut server = Server(
            Command::new(env!("CARGO_BIN_EXE_revault_key_server"))
                .args(["run", "--config"])
                .arg(&config)
                .env(
                    "REVAULT_KEY_SERVER_LOG",
                    directory.path().join("server.log"),
                )
                .stdout(Stdio::null())
                .stderr(std::fs::File::create(&errors).unwrap())
                .spawn()
                .unwrap(),
        );
        let response = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    assert!(server.0.try_wait().unwrap().is_none(), "HTTPS server exited: {}", std::fs::read_to_string(&errors).unwrap());
                    if let Ok(socket) = tokio::net::TcpStream::connect(addr).await {
                        let name = rustls::pki_types::ServerName::try_from("keyshare.test").unwrap();
                        if let Ok(mut tls) = connector.connect(name, socket).await {
                            assert_eq!(tls.get_ref().1.peer_certificates().unwrap()[0], *cert.cert.der());
                            tls.write_all(b"GET /v1/status HTTP/1.1\r\nHost: keyshare.test\r\nConnection: close\r\n\r\n").await.unwrap();
                            let mut bytes = Vec::new();
                            match tls.read_to_end(&mut bytes).await {
                                Ok(_) => {},
                                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {},
                                Err(error) => panic!("HTTPS response failed: {error}"),
                            }
                            break bytes;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }).await.expect("HTTPS startup timed out")
        });
        assert!(response.starts_with(b"HTTP/1.1 200"));
        let body = response
            .windows(4)
            .position(|part| part == b"\r\n\r\n")
            .unwrap()
            + 4;
        assert!(
            response[body..].starts_with(b"LBSS"),
            "expected binary key-server status over TLS"
        );
        drop(server);
    }
}
