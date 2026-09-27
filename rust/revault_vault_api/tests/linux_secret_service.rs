#![cfg(target_os = "linux")]
//! Public API integration against a private bus and synthetic credential provider.
//! No real desktop service, user credentials or production crypto parameters change.
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Read,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use zbus::{
    blocking::connection::Builder,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};
const SERVICE_PATH: &str = "/org/freedesktop/secrets";
const COLLECTION: &str = "/org/freedesktop/secrets/collection/test";
const ITEM: &str = "/org/freedesktop/secrets/collection/test/item";
const SECRET: &str = "synthetic private-bus Vault credential";
type WireSecret = (OwnedObjectPath, Vec<u8>, Vec<u8>, String);
fn path(value: &str) -> OwnedObjectPath {
    value.try_into().unwrap()
}
struct State {
    mode: String,
    secret: Mutex<Option<Vec<u8>>>,
    attributes: Mutex<Option<HashMap<String, String>>>,
    keys: Mutex<BTreeMap<String, [u8; 16]>>,
}
impl State {
    fn decode(&self, secret: WireSecret) -> Vec<u8> {
        let key = self.keys.lock().unwrap()[secret.0.as_str()];
        cbc::Decryptor::<aes::Aes128>::new((&key).into(), secret.1.as_slice().into())
            .decrypt_padded_vec_mut::<Pkcs7>(&secret.2)
            .unwrap()
    }
}
struct Service(Arc<State>);
#[zbus::interface(name = "org.freedesktop.Secret.Service")]
impl Service {
    async fn open_session(
        &self,
        algorithm: &str,
        input: Value<'_>,
    ) -> zbus::fdo::Result<(OwnedValue, OwnedObjectPath)> {
        assert_eq!(algorithm, "dh-ietf1024-sha256-aes128-cbc-pkcs7");
        if self.0.mode == "negotiation-hang" {
            std::future::pending::<()>().await;
        }
        let peer: Vec<u8> = input.try_into().unwrap();
        assert!(!peer.is_empty() && peer.len() <= 128);
        // Synthetic server exponent=1, public=2: shared secret is the client's
        // public value. This deliberately trivial TEST key permits an independent
        // CBC/HKDF oracle without changing the production DH implementation.
        let mut shared = [0u8; 128];
        shared[128 - peer.len()..].copy_from_slice(&peer);
        let mut key = [0u8; 16];
        hkdf::Hkdf::<sha2::Sha256>::new(None, &shared)
            .expand(&[], &mut key)
            .unwrap();
        let mut keys = self.0.keys.lock().unwrap();
        let session = format!("/org/freedesktop/secrets/session/s{}", keys.len());
        keys.insert(session.clone(), key);
        Ok((
            OwnedValue::try_from(Value::from(vec![2u8])).unwrap(),
            path(&session),
        ))
    }
    async fn search_items(
        &self,
        attributes: HashMap<String, String>,
    ) -> zbus::fdo::Result<(Vec<OwnedObjectPath>, Vec<OwnedObjectPath>)> {
        assert_eq!(
            attributes.get("service").unwrap(),
            "dev.onepub.lockbox.vault"
        );
        assert!(attributes
            .get("username")
            .unwrap()
            .ends_with("local-vault.lbox"));
        if self.0.mode == "denied" {
            return Err(zbus::fdo::Error::AccessDenied(
                "synthetic denied credential store".into(),
            ));
        }
        if self.0.mode == "method-hang" {
            std::future::pending::<()>().await;
        }
        if self.0.mode == "locked" {
            return Ok((vec![], vec![path(ITEM)]));
        }
        let found = self.0.secret.lock().unwrap().is_some()
            && self
                .0
                .attributes
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|saved| attributes.iter().all(|(k, v)| saved.get(k) == Some(v)));
        Ok((if found { vec![path(ITEM)] } else { vec![] }, vec![]))
    }
    fn read_alias(&self, alias: &str) -> OwnedObjectPath {
        assert_eq!(alias, "default");
        path(COLLECTION)
    }
    fn unlock(&self, _objects: Vec<OwnedObjectPath>) -> (Vec<OwnedObjectPath>, OwnedObjectPath) {
        panic!("auto-open must not request an unlock prompt")
    }
}
struct Collection(Arc<State>);
#[zbus::interface(name = "org.freedesktop.Secret.Collection")]
impl Collection {
    #[zbus(property)]
    fn locked(&self) -> bool {
        false
    }
    fn create_item(
        &self,
        mut properties: HashMap<String, OwnedValue>,
        secret: WireSecret,
        replace: bool,
    ) -> (OwnedObjectPath, OwnedObjectPath) {
        assert!(replace);
        if self.0.mode == "prompt-hang" {
            return (path("/"), path("/org/freedesktop/secrets/prompt/p"));
        }
        let attributes: HashMap<String, String> = properties
            .remove("org.freedesktop.Secret.Item.Attributes")
            .unwrap()
            .try_into()
            .unwrap();
        *self.0.attributes.lock().unwrap() = Some(attributes);
        *self.0.secret.lock().unwrap() = Some(self.0.decode(secret));
        (path(ITEM), path("/"))
    }
}
struct Item(Arc<State>);
#[zbus::interface(name = "org.freedesktop.Secret.Item")]
impl Item {
    #[zbus(property)]
    fn locked(&self) -> bool {
        self.0.mode == "locked"
    }
    fn get_secret(&self, session: OwnedObjectPath) -> WireSecret {
        let key = self.0.keys.lock().unwrap()[session.as_str()];
        let iv = [0x23; 16]; // Synthetic test ciphertext only.
        let guard = self.0.secret.lock().unwrap();
        let secret = guard.as_ref().unwrap();
        let cipher = cbc::Encryptor::<aes::Aes128>::new((&key).into(), (&iv).into())
            .encrypt_padded_vec_mut::<Pkcs7>(secret);
        (
            session,
            iv.to_vec(),
            cipher,
            "application/octet-stream".into(),
        )
    }
    fn set_secret(&self, secret: WireSecret) {
        *self.0.secret.lock().unwrap() = Some(self.0.decode(secret));
    }
    fn delete(&self) -> OwnedObjectPath {
        *self.0.secret.lock().unwrap() = None;
        path("/")
    }
}
struct Prompt;
#[zbus::interface(name = "org.freedesktop.Secret.Prompt")]
impl Prompt {
    fn prompt(&self, window_id: &str) {
        assert!(window_id.is_empty());
    }
    // Deliberately never emits Completed: method timeout alone cannot bound this.
}
struct PrivateBus {
    child: Child,
    directory: PathBuf,
}
impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.directory);
    }
}
fn run(mode: &str) {
    let directory = std::env::temp_dir().join(format!(
        "lbx-secret-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let address = format!("unix:path={}", directory.join("bus").display());
    let child = Command::new("dbus-daemon")
        .args(["--session", "--nofork", &format!("--address={address}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("install dbus-daemon for private-bus integration tests");
    let bus = PrivateBus { child, directory };
    let started = Instant::now();
    let ready = loop {
        match Builder::address(address.as_str())
            .unwrap()
            .method_timeout(Duration::from_secs(2))
            .build()
        {
            Ok(connection) => break connection,
            Err(error) => {
                assert!(started.elapsed() < Duration::from_secs(5), "{error}");
                thread::sleep(Duration::from_millis(10));
            }
        }
    };
    let state = Arc::new(State {
        mode: mode.into(),
        secret: Mutex::new(None),
        attributes: Mutex::new(None),
        keys: Mutex::new(BTreeMap::new()),
    });
    let _service = Builder::address(address.as_str())
        .unwrap()
        .name("org.freedesktop.secrets")
        .unwrap()
        .serve_at(SERVICE_PATH, Service(state.clone()))
        .unwrap()
        .serve_at(COLLECTION, Collection(state.clone()))
        .unwrap()
        .serve_at(ITEM, Item(state))
        .unwrap()
        .serve_at("/org/freedesktop/secrets/prompt/p", Prompt)
        .unwrap()
        .build()
        .unwrap();
    drop(ready);
    let handshake_path = bus.directory.join("hung-handshake");
    let _handshake = (mode == "handshake-hang")
        .then(|| std::os::unix::net::UnixListener::bind(&handshake_path).unwrap());
    let address = if _handshake.is_some() {
        format!("unix:path={}", handshake_path.display())
    } else {
        address
    };
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "credential_store_child", "--nocapture"])
        .env("REVAULT_SECRET_SERVICE_SCENARIO", mode)
        .env("LOCKBOX_VAULT_DIR", bus.directory.join("vault"))
        .env("DBUS_SESSION_BUS_ADDRESS", &address)
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "auto")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let mut output = String::new();
            child
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut output)
                .unwrap();
            child
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut output)
                .unwrap();
            assert!(status.success(), "{mode}: {output}");
            assert!(!output.contains(SECRET));
            break;
        }
        if started.elapsed() > Duration::from_secs(15) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("credential operation hung: {mode}");
        }
        thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn private_secret_service_round_trips_existing_credential_attributes() {
    run("healthy");
}
#[test]
fn locked_store_fails_without_prompting() {
    run("locked");
}
#[test]
fn denied_store_is_an_actionable_error() {
    run("denied");
}
#[test]
fn hung_method_has_a_total_deadline() {
    run("method-hang");
}
#[test]
fn hung_session_negotiation_has_a_total_deadline() {
    run("negotiation-hang");
}
#[test]
fn credentials_remain_bidirectionally_compatible_with_keyring_backend() {
    run("legacy");
}
#[test]
fn hung_prompt_is_bounded_even_after_its_method_returns() {
    run("prompt-hang");
}
#[test]
fn hung_bus_handshake_is_bounded() {
    run("handshake-hang");
}
#[test]
fn credential_store_child() {
    let Ok(mode) = std::env::var("REVAULT_SECRET_SERVICE_SCENARIO") else {
        return;
    };
    use revault_lockbox_api::SecretString;
    use revault_vault_api::{
        forget_platform_vault_password, get_platform_vault_password,
        get_platform_vault_password_for, put_platform_vault_password,
    };
    let started = Instant::now();
    if mode == "healthy" || mode == "legacy" {
        assert!(get_platform_vault_password().unwrap().is_none());
        let secret = SecretString::try_from_slice(SECRET.as_bytes()).unwrap();
        let item =
            PathBuf::from(std::env::var_os("LOCKBOX_VAULT_DIR").unwrap()).join("local-vault.lbox");
        if mode == "legacy" {
            keyring::Entry::new("dev.onepub.lockbox.vault", &item.to_string_lossy())
                .unwrap()
                .set_secret(SECRET.as_bytes())
                .unwrap();
        } else {
            put_platform_vault_password(&secret).unwrap();
        }
        let got = get_platform_vault_password().unwrap().unwrap();
        got.with_str(|value| assert!(value == SECRET)).unwrap();
        let replacement = SecretString::try_from_slice(b"synthetic replacement").unwrap();
        put_platform_vault_password(&replacement).unwrap();
        if mode == "legacy" {
            let bytes = keyring::Entry::new("dev.onepub.lockbox.vault", &item.to_string_lossy())
                .unwrap()
                .get_secret()
                .unwrap();
            assert!(bytes == b"synthetic replacement");
        }
        let directory = PathBuf::from(std::env::var_os("LOCKBOX_VAULT_DIR").unwrap());
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap();
        get_platform_vault_password_for(&directory, Some(&address))
            .unwrap()
            .unwrap()
            .with_str(|value| assert!(value == "synthetic replacement"))
            .unwrap();
        let invalid_address = format!(
            "unix:path={}",
            directory.join("missing-explicit-bus").display()
        );
        assert!(get_platform_vault_password_for(&directory, Some(&invalid_address)).is_err());
        get_platform_vault_password()
            .unwrap()
            .unwrap()
            .with_str(|value| assert!(value == "synthetic replacement"))
            .unwrap();
        forget_platform_vault_password().unwrap();
        forget_platform_vault_password().unwrap();
        assert!(get_platform_vault_password().unwrap().is_none());
    } else {
        let result = if mode == "prompt-hang" {
            put_platform_vault_password(&SecretString::try_from_slice(SECRET.as_bytes()).unwrap())
        } else {
            get_platform_vault_password().map(|_| ())
        };
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected credential-store failure"),
        };
        let message = error.to_string();
        assert!(message.contains("platform credential store"));
        assert!(message.contains("explicit Vault credentials"));
        if mode == "locked" || mode == "denied" {
            assert!(put_platform_vault_password(
                &SecretString::try_from_slice(SECRET.as_bytes()).unwrap()
            )
            .is_err());
            assert!(forget_platform_vault_password().is_err());
        }
        if mode.ends_with("hang") {
            assert!(started.elapsed() >= Duration::from_secs(4));
        }
        assert!(started.elapsed() < Duration::from_secs(8));
    }
}
