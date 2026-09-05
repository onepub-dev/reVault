//! E2E state is created and verified through public CLI commands.
//! The HTTP relay is launched through its public listener API because Cargo
//! does not provide another package's binary path to this integration test.
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;

struct User {
    vault: PathBuf,
    agent: PathBuf,
}
impl User {
    fn new(root: &Path, name: &str, email: &str) -> Self {
        let user = Self {
            vault: root.join(format!("{name}v")),
            agent: root.join(format!("{name}a")),
        };
        user.ok(&["vault", "init"]);
        user.ok(&["vault", "profile", "email", "default", email]);
        user
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .args(args)
            .env("LOCKBOX_VAULT_DIR", &self.vault)
            .env("LOCKBOX_SESSION_AGENT_DIR", &self.agent)
            .env("LOCKBOX_SESSION_AGENT_LOG", self.agent.join("agent.log"))
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_VAULT_PASSWORD", "exchange-test-passphrase")
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("LOCKBOX_CONTENT_KEY")
            .output()
            .unwrap()
    }
    fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
    fn fails(&self, args: &[&str]) {
        let output = self.run(args);
        assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
    }
}
impl Drop for User {
    fn drop(&mut self) {
        let _ = self.run(&["session", "stop"]);
    }
}

fn field(output: &str, name: &str) -> String {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .unwrap_or_else(|| panic!("missing {name}: {output}"))
        .to_owned()
}

fn server(root: &Path) -> String {
    let store =
        revault_key_server::store::PublishStore::open(revault_key_server::store::ServerConfig {
            state_dir: root.join("server"),
            ..Default::default()
        })
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        revault_key_server::server::run_listener(listener, Arc::new(store)).unwrap()
    });
    format!("http://{address}")
}

fn exchange(alice: &User, bob: &User, server: &str) -> (String, String) {
    let offered = alice.ok(&[
        "vault",
        "contact",
        "exchange",
        "bob@example.test",
        "--profile",
        "default",
        "--key-server",
        server,
    ]);
    let invitation = field(&offered, "invitation");
    let id = field(&offered, "exchange");
    let accepted = bob.ok(&[
        "vault",
        "contact",
        "accept",
        &invitation,
        "--profile",
        "default",
    ]);
    let code = field(&accepted, "verification");
    let repeat = bob.ok(&[
        "vault",
        "contact",
        "accept",
        &invitation,
        "--profile",
        "default",
    ]);
    assert_eq!(code, field(&repeat, "verification"));
    let fetched = alice.ok(&["vault", "contact", "exchanges"]);
    // May include prior exchanges; choose the block for this invitation.
    let block = fetched.split(&format!("exchange={id}")).nth(1).unwrap();
    assert_eq!(code, field(block, "verification"));
    (id, code)
}

fn verify(user: &User, id: &str, contact: &str, code: &str) {
    user.ok(&[
        "vault",
        "contact",
        "verify",
        id,
        contact,
        "--fingerprint",
        code,
        "--channel",
        "in-person",
    ]);
}

fn exchange_archive(
    sender: &User,
    receiver: &User,
    root: &Path,
    label: &str,
    recipient: &str,
    author: &str,
) {
    let archive = root.join(format!("{label}.lbox"));
    let source = root.join(format!("{label}.bin"));
    let bytes = b"reciprocal exchange\0persisted content\xff\n";
    // Host input content is not lockbox/vault internals.
    std::fs::write(&source, bytes).unwrap();
    let path = archive.to_str().unwrap();
    sender.ok(&[path, "create"]);
    sender.ok(&[path, "open"]);
    sender.ok(&[
        path,
        "add",
        source.to_str().unwrap(),
        "--to",
        "/payload.bin",
    ]);
    sender.ok(&[path, "access", "grant", recipient]);
    receiver.ok(&[path, "open"]);
    let extracted = receiver.run(&[path, "cat", "/payload.bin"]);
    assert!(
        extracted.status.success(),
        "{}",
        String::from_utf8_lossy(&extracted.stderr)
    );
    assert_eq!(extracted.stdout, bytes);
    receiver.ok(&["vault", "contact", "verify-author", author, path]);
}

#[test]
fn reciprocal_exchange_verifies_both_keys_and_real_archives_in_both_directions() {
    let root = tempfile::Builder::new()
        .prefix("lbx-ex-")
        .tempdir()
        .unwrap();
    let server = server(root.path());
    let alice = User::new(root.path(), "a", "alice@example.test");
    let bob = User::new(root.path(), "b", "bob@example.test");
    let (id, code) = exchange(&alice, &bob, &server);
    assert!(!alice.ok(&["vault", "contact", "list"]).contains("bob"));
    assert!(!bob.ok(&["vault", "contact", "list"]).contains("alice"));
    bob.fails(&[
        "vault",
        "contact",
        "verify",
        &id,
        "alice",
        "--fingerprint",
        "000000",
        "--channel",
        "in-person",
    ]);
    assert!(!bob.ok(&["vault", "contact", "list"]).contains("alice"));
    verify(&alice, &id, "bob", &code);
    // Alice's confirmation must not set Bob's trust state.
    assert!(!bob.ok(&["vault", "contact", "list"]).contains("alice"));
    verify(&bob, &id, "alice", &code);
    verify(&bob, &id, "alice", &code);
    assert!(alice.ok(&["vault", "contact", "list"]).contains("bob"));
    assert!(bob.ok(&["vault", "contact", "list"]).contains("alice"));
    exchange_archive(&alice, &bob, root.path(), "alice-to-bob", "bob", "alice");
    exchange_archive(&bob, &alice, root.path(), "bob-to-alice", "alice", "bob");
    // New exchange of the same identities is safe and repeatable.
    let (second, code) = exchange(&alice, &bob, &server);
    verify(&alice, &second, "bob", &code);
    verify(&bob, &second, "alice", &code);
    alice.ok(&["vault", "contact", "forget-exchange", &id]);
    alice.ok(&["vault", "contact", "forget-exchange", &id]);
    assert!(!alice
        .ok(&["vault", "contact", "exchanges", "--offline"])
        .contains(&id));
    assert!(alice.ok(&["vault", "contact", "list"]).contains("bob"));
}

#[test]
fn recipient_selection_cancellation_and_changed_identity_refusals() {
    let root = tempfile::Builder::new()
        .prefix("lbx-ex-")
        .tempdir()
        .unwrap();
    let server = server(root.path());
    let alice = User::new(root.path(), "a", "alice@example.test");
    let bob = User::new(root.path(), "b", "bob@example.test");
    let mallory = User::new(root.path(), "m", "mallory@example.test");
    let output = alice.ok(&[
        "vault",
        "contact",
        "exchange",
        "bob@example.test",
        "--profile",
        "default",
        "--key-server",
        &server,
    ]);
    let id = field(&output, "exchange");
    let invitation = field(&output, "invitation");
    mallory.fails(&[
        "vault",
        "contact",
        "accept",
        &invitation,
        "--profile",
        "default",
    ]);
    bob.fails(&[
        "vault",
        "contact",
        "accept",
        &invitation,
        "--profile",
        "default",
        "--receive-only",
    ]);
    alice.ok(&["vault", "contact", "cancel-exchange", &id]);
    assert!(!alice
        .ok(&["vault", "contact", "exchanges", "--offline"])
        .contains(&id));
    bob.fails(&[
        "vault",
        "contact",
        "accept",
        &invitation,
        "--profile",
        "default",
    ]);
    let (id, code) = exchange(&alice, &bob, &server);
    verify(&alice, &id, "bob", &code);
    verify(&bob, &id, "alice", &code);
    alice.fails(&["vault", "contact", "cancel-exchange", &id]);
    // A different profile claiming the same email still needs fresh verification
    // and must not silently replace the established contact.
    mallory.ok(&["vault", "profile", "email", "default", "bob@example.test"]);
    let (changed, changed_code) = exchange(&alice, &mallory, &server);
    alice.fails(&[
        "vault",
        "contact",
        "verify",
        &changed,
        "bob",
        "--fingerprint",
        &changed_code,
        "--channel",
        "in-person",
    ]);
    exchange_archive(
        &alice,
        &bob,
        root.path(),
        "still-original-bob",
        "bob",
        "alice",
    );
    alice.ok(&["vault", "contact", "remove", "bob"]);
    assert!(!alice.ok(&["vault", "contact", "list"]).contains("bob"));
    verify(&alice, &changed, "bob", &changed_code);
    verify(&mallory, &changed, "alice", &changed_code);
    exchange_archive(
        &alice,
        &mallory,
        root.path(),
        "explicit-replacement",
        "bob",
        "alice",
    );
    bob.fails(&[
        root.path()
            .join("explicit-replacement.lbox")
            .to_str()
            .unwrap(),
        "open",
    ]);
}
