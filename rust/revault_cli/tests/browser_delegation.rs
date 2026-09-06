#![cfg(all(target_os = "linux", feature = "browser-integration"))]
mod common;
use common::TestTempDir;
use ed25519_dalek::{Signer, SigningKey};
use revault_browser_protocol::{
    crypto::seal,
    receiver::{Application, Receiver},
    Message,
};
use revault_lockbox_api::{vault_integration::VaultOpen, LockboxPath, SecretString};
use std::{fs, io::Read, process::Command};

#[test]
fn synthetic_profile_lockbox_unlock_restart_replay_and_new_approval() {
    let temp = TestTempDir::new("browser-receiver");
    let root = temp.path();
    let vault = root.join("vault");
    let agent = tempfile::tempdir().unwrap();
    struct AgentGuard(std::path::PathBuf);
    impl Drop for AgentGuard {
        fn drop(&mut self) {
            let _ = Command::new(env!("CARGO_BIN_EXE_lockbox"))
                .args(["session", "stop"])
                .env("LOCKBOX_SESSION_AGENT_DIR", &self.0)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
    let _agent_guard = AgentGuard(agent.path().to_path_buf());
    let lockbox = root.join("synthetic.lbox");
    let source = root.join("fixture.bin");
    let secret = b"synthetic diagnostic credential\0synthetic beta credential\xff";
    fs::write(&source, secret).unwrap();
    let cli = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_lockbox"))
            .args(args)
            .env("LOCKBOX_VAULT_DIR", &vault)
            .env("LOCKBOX_VAULT_PASSWORD", "synthetic vault test password")
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env("LOCKBOX_SESSION_AGENT_DIR", agent.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    cli(&["vault", "init"]);
    cli(&[lockbox.to_str().unwrap(), "create"]);
    cli(&[lockbox.to_str().unwrap(), "open"]);
    cli(&[
        lockbox.to_str().unwrap(),
        "add",
        source.to_str().unwrap(),
        "--to",
        "/synthetic.bin",
    ]);
    // Separately invoke the public CLI and compare the persisted content bytes.
    assert_eq!(
        cli(&[lockbox.to_str().unwrap(), "cat", "/synthetic.bin"]),
        secret
    );
    // Exception: the public CLI intentionally cannot export a raw content key.
    // Exercise the narrow profile-grant API at the agent/HPKE test boundary.
    // All vault/lockbox setup and persisted-state verification above use the CLI.
    let password = SecretString::try_from_slice(b"synthetic vault test password").unwrap();
    let local =
        revault_vault_api::VaultDirectory::open_file(vault.join("local-vault.lbox"), &password)
            .unwrap();
    let profile = local.load_private_key("default").unwrap();
    let opened = VaultOpen::path_with_contact(&lockbox, &profile).unwrap();
    let key = opened.try_clone_key().unwrap();
    let app = Application {
        origin: "https://synthetic.example".into(),
        application_id: "synthetic-receiver".into(),
        lockbox_id: opened.lockbox_id.to_string(),
    };
    let signing = SigningKey::from_bytes(&[17; 32]);
    let session = [23; 32];
    let mut server = Receiver::new().unwrap();
    let request = server
        .issue(&app, session, 1000, |b| Ok(signing.sign(b).to_bytes()))
        .unwrap();
    // Approval is represented by the agent-side seal operation here; native UI
    // denial/cancellation and authorisation are tested in the policy suite.
    let envelope = seal(&key, request.request.as_ref().unwrap()).unwrap();
    let browser_traffic = envelope.encode_to_vec();
    assert!(!browser_traffic.windows(secret.len()).any(|b| b == secret));
    assert!(!key
        .with_bytes(|bytes| browser_traffic.windows(bytes.len()).any(|b| b == bytes))
        .unwrap());
    let handle = server
        .accept(&session, &envelope, 1001)
        .unwrap()
        .open_lockbox(&lockbox)
        .unwrap();
    let mut actual = Vec::new();
    handle
        .open_file(&LockboxPath::new("/synthetic.bin").unwrap())
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(actual, secret);
    // Restart drops every handle and receiver. No server Session Agent is used.
    drop(handle);
    drop(server);
    let mut server = Receiver::new().unwrap();
    assert!(server.accept(&session, &envelope, 1002).is_err());
    let fresh = server
        .issue(&app, session, 1002, |b| Ok(signing.sign(b).to_bytes()))
        .unwrap();
    let envelope = seal(&key, fresh.request.as_ref().unwrap()).unwrap();
    let handle = server
        .accept(&session, &envelope, 1003)
        .unwrap()
        .open_lockbox(&lockbox)
        .unwrap();
    actual.clear();
    handle
        .open_file(&LockboxPath::new("/synthetic.bin").unwrap())
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(actual, secret);
}

#[test]
fn native_installation_replacement_removal_and_transport() {
    use revault_browser_protocol::{
        browser_request::Operation,
        response,
        transport::{read_frame, write_frame},
        BrowserRequest, BrowserResponse, Error,
    };
    use std::{
        io::{Cursor, Write},
        process::Stdio,
    };
    let temp = TestTempDir::new("browser-installer");
    let root = temp.path();
    let short = tempfile::tempdir().unwrap();
    let chrome = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let firefox = "browser@revault.onepub.dev";
    let host = env!("CARGO_BIN_EXE_revault-browser");
    let command = |args: &[&str]| {
        let mut cmd = Command::new(host);
        cmd.args(args)
            .env("REVAULT_BROWSER_INSTALL_ROOT", root)
            .env("LOCKBOX_VAULT_DIR", root.join("vault"))
            .env("LOCKBOX_SESSION_AGENT_DIR", short.path());
        cmd
    };
    let success = |args: &[&str]| {
        let result = command(args).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    };
    assert_eq!(success(&["hosts"]), b"[]\n");
    success(&["install", chrome, firefox]);
    let installed = success(&["hosts"]);
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&installed).unwrap();
    assert_eq!(entries.len(), 3);
    for entry in entries {
        let manifest = &entry["manifest"];
        assert_eq!(
            manifest["path"],
            fs::canonicalize(host).unwrap().to_str().unwrap()
        );
        if manifest.get("allowed_origins").is_some() {
            assert_eq!(
                manifest["allowed_origins"],
                serde_json::json!([format!("chrome-extension://{chrome}/")])
            );
        } else {
            assert_eq!(manifest["allowed_extensions"], serde_json::json!([firefox]));
        }
    }
    success(&["install", chrome, firefox]);
    assert_eq!(success(&["hosts"]), installed);
    let invalid = command(&["install", "*", firefox]).output().unwrap();
    assert!(!invalid.status.success());
    assert_eq!(success(&["hosts"]), installed);
    let origin = format!("chrome-extension://{chrome}/");
    let request = BrowserRequest {
        protocol_version: 1,
        origin: "https://synthetic.example".into(),
        extension_id: chrome.into(),
        frame_id: 0,
        operation: Some(Operation::GetCapabilities(true)),
    };
    let send = |args: &[&str], frame: &[u8]| {
        let mut child = command(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(frame).unwrap();
        child.wait_with_output().unwrap()
    };
    let mut frame = Vec::new();
    write_frame(&mut frame, &request).unwrap();
    let result = send(&[&origin], &frame);
    assert!(result.status.success());
    assert!(result.stderr.is_empty());
    let bytes = read_frame(&mut Cursor::new(result.stdout))
        .unwrap()
        .unwrap();
    assert_eq!(
        BrowserResponse::decode(bytes.as_slice()).unwrap(),
        response(Err(Error::NativeHelperMissing))
    );
    let result = send(&[&origin], &(u32::MAX).to_ne_bytes());
    assert!(result.status.success());
    assert!(result.stderr.is_empty());
    let bytes = read_frame(&mut Cursor::new(result.stdout))
        .unwrap()
        .unwrap();
    assert_eq!(
        BrowserResponse::decode(bytes.as_slice()).unwrap(),
        response(Err(Error::InvalidRequest))
    );
    let wrong_origin = "chrome-extension://bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/";
    let result = send(&[wrong_origin], &[]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    success(&["install", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", firefox]);
    assert_ne!(success(&["hosts"]), installed);
    assert!(!send(&[&origin], &[]).status.success());
    success(&["uninstall"]);
    assert_eq!(success(&["hosts"]), b"[]\n");
    success(&["uninstall"]);
    assert_eq!(success(&["hosts"]), b"[]\n");
}
