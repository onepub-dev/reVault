use super::*;
use ed25519_dalek::{Signer, SigningKey};
use revault_browser_protocol::{
    crypto::RecipientKey, validation::signing_bytes, SignedUnlockRequest, UnlockRequest, SCOPE,
};
use std::sync::atomic::AtomicUsize;
static NEXT: AtomicUsize = AtomicUsize::new(0);
static SERIAL: Mutex<()> = Mutex::new(());
struct Fixture {
    _serial: std::sync::MutexGuard<'static, ()>,
    state: Arc<State>,
    request: BrowserRequest,
    pair: Pairing,
    calls: Arc<AtomicUsize>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.state.root);
    }
}
fn fixture(approval: Arc<Approval>, key_error: Option<Error>) -> Fixture {
    let serial = SERIAL.lock().unwrap();
    let root = std::env::temp_dir().join(format!(
        "revault-browser-policy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    secure_directory(&root).unwrap();
    let signing = SigningKey::from_bytes(&[8; 32]);
    let pair = Pairing {
        origin: "https://test.example".into(),
        application_id: "test".into(),
        application_name: "Synthetic receiver".into(),
        lockbox_id: "lockbox-1".into(),
        lockbox_path: root.join("synthetic.lbox"),
        profile: "default".into(),
        server_signing_key: signing.verifying_key().to_bytes(),
        extension_id: "test@revault".into(),
        disclosure: "Contains synthetic credentials only".into(),
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(format!("{}.json", pair.id())))
        .unwrap();
    file.write_all(&serde_json::to_vec(&pair).unwrap()).unwrap();
    let request = UnlockRequest {
        protocol_version: VERSION,
        request_id: "test-request".into(),
        application_id: pair.application_id.clone(),
        server_boot_id: vec![1; 32],
        challenge: vec![2; 32],
        issued_at: now().unwrap(),
        expires_at: now().unwrap() + 120,
        recipient_key_id: "boot-1".into(),
        recipient_public_key: RecipientKey::generate().unwrap().public_key().to_vec(),
        lockbox_id: pair.lockbox_id.clone(),
        requested_scope: SCOPE.into(),
        origin: pair.origin.clone(),
    };
    let signature = signing.sign(&signing_bytes(&request)).to_bytes().to_vec();
    let request = BrowserRequest {
        protocol_version: VERSION,
        origin: pair.origin.clone(),
        extension_id: pair.extension_id.clone(),
        frame_id: 0,
        operation: Some(Operation::RequestUnlock(SignedUnlockRequest {
            request: Some(request),
            signature,
        })),
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let state = Arc::new(State {
        root,
        pending: Mutex::new(BTreeMap::new()),
        cancelled: Mutex::new(BTreeMap::new()),
        approval,
        replay: Mutex::new(Replay::default()),
        keys: Arc::new(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(error) = key_error {
                return Err(error);
            }
            SecretVec::try_from_slice(&[77; 32]).map_err(|_| Error::Internal)
        }),
        stopped: Arc::new(AtomicBool::new(false)),
    });
    Fixture {
        _serial: serial,
        state,
        request,
        pair,
        calls,
    }
}
#[test]
fn wrong_context_never_shows_approval_or_reads_key() {
    let shown = Arc::new(AtomicUsize::new(0));
    let count = shown.clone();
    let f = fixture(
        Arc::new(move |_, _, _| {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
        None,
    );
    let mut requests = Vec::new();
    let mut r = f.request.clone();
    r.origin = "https://evil.example".into();
    requests.push(r);
    let mut r = f.request.clone();
    r.extension_id = "wrong@extension".into();
    requests.push(r);
    let mut r = f.request.clone();
    r.frame_id = 1;
    requests.push(r);
    let mut r = f.request.clone();
    r.protocol_version += 1;
    requests.push(r);
    let mut r = f.request.clone();
    if let Some(Operation::RequestUnlock(s)) = &mut r.operation {
        s.request.as_mut().unwrap().lockbox_id = "other-lockbox".into();
    }
    requests.push(r);
    for r in requests {
        assert!(f.state.handle(r).is_err());
    }
    assert_eq!(shown.load(Ordering::SeqCst), 0);
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn denial_errors_and_persisted_replay_are_fail_closed() {
    for error in [
        Error::VaultLocked,
        Error::LockboxUnavailable,
        Error::SecureStoreUnavailable,
    ] {
        let f = fixture(Arc::new(|_, _, _| Ok(())), Some(error));
        assert_eq!(f.state.handle(f.request.clone()), Err(error));
        assert_eq!(f.state.handle(f.request.clone()), Err(Error::Replayed));
        // A new in-memory ledger loaded after restart still rejects replay.
        let restored: Replay =
            serde_json::from_slice(&fs::read(f.state.root.join("replay.state")).unwrap()).unwrap();
        *f.state.replay.lock().unwrap() = restored;
        assert_eq!(f.state.handle(f.request.clone()), Err(Error::Replayed));
    }
    let f = fixture(Arc::new(|_, _, _| Err(Error::Denied)), None);
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Denied));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn cancellation_before_and_during_approval() {
    let f = fixture(Arc::new(|_, _, _| Ok(())), None);
    let mut cancel = f.request.clone();
    cancel.operation = Some(Operation::CancelRequest("test-request".into()));
    assert_eq!(f.state.handle(cancel), Ok(None));
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Cancelled));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
    drop(f);
    let shown = Arc::new(AtomicBool::new(false));
    let flag = shown.clone();
    let f = fixture(
        Arc::new(move |_, cancel, _| {
            flag.store(true, Ordering::SeqCst);
            while !cancel.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(1));
            }
            Err(Error::Cancelled)
        }),
        None,
    );
    let state = f.state.clone();
    let request = f.request.clone();
    let worker = thread::spawn(move || state.handle(request));
    while !shown.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Replayed));
    let mut cancel = f.request.clone();
    cancel.operation = Some(Operation::CancelRequest("test-request".into()));
    assert_eq!(f.state.handle(cancel), Ok(None));
    assert_eq!(worker.join().unwrap(), Err(Error::Cancelled));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn revocation_while_approval_is_open_prevents_release() {
    let f = fixture(Arc::new(|_, _, _| Ok(())), None);
    let root = f.state.root.clone();
    let id = f.pair.id();
    // Replace via a separate State; fixture remains responsible for cleanup.
    let test = State {
        root: root.clone(),
        pending: Mutex::new(BTreeMap::new()),
        cancelled: Mutex::new(BTreeMap::new()),
        replay: Mutex::new(Replay::default()),
        approval: Arc::new(move |_, _, _| {
            fs::remove_file(root.join(format!("{id}.json"))).unwrap();
            Ok(())
        }),
        keys: f.state.keys.clone(),
        stopped: f.state.stopped.clone(),
    };
    assert_eq!(
        test.handle(f.request.clone()),
        Err(Error::UnpairedRecipient)
    );
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn replay_clock_rollback_fails_closed() {
    let f = fixture(Arc::new(|_, _, _| Ok(())), None);
    f.state.replay.lock().unwrap().high_water = now().unwrap() + 1;
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Expired));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}

fn numbered_request(base: &BrowserRequest, number: u8) -> BrowserRequest {
    let mut request = base.clone();
    if let Some(Operation::RequestUnlock(signed)) = &mut request.operation {
        let binding = signed.request.as_mut().unwrap();
        binding.request_id = format!("request-{number}");
        binding.challenge = vec![number; 32];
        signed.signature = SigningKey::from_bytes(&[8; 32])
            .sign(&signing_bytes(binding))
            .to_bytes()
            .to_vec();
    }
    request
}
#[test]
fn concurrent_limit_keeps_cancellation_available() {
    let shown = Arc::new(AtomicUsize::new(0));
    let counter = shown.clone();
    let f = fixture(
        Arc::new(move |_, cancel, _| {
            counter.fetch_add(1, Ordering::SeqCst);
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while !cancel.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
                thread::sleep(Duration::from_millis(1));
            }
            Err(Error::Cancelled)
        }),
        None,
    );
    let workers: Vec<_> = (0..8)
        .map(|i| {
            let state = f.state.clone();
            let request = numbered_request(&f.request, i);
            thread::spawn(move || state.handle(request))
        })
        .collect();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while shown.load(Ordering::SeqCst) != 8 && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(shown.load(Ordering::SeqCst), 8);
    assert_eq!(
        f.state.handle(numbered_request(&f.request, 9)),
        Err(Error::Busy)
    );
    for i in 0..8 {
        let mut cancel = f.request.clone();
        cancel.operation = Some(Operation::CancelRequest(format!("request-{i}")));
        assert_eq!(f.state.handle(cancel), Ok(None));
    }
    for worker in workers {
        assert_eq!(worker.join().unwrap(), Err(Error::Cancelled));
    }
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn suspend_during_approval_invalidates_release() {
    let f = fixture(
        Arc::new(|_, _, _| {
            crate::browser::suspend();
            Ok(())
        }),
        None,
    );
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Cancelled));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn successful_release_persists_no_content_key_or_plaintext() {
    let f = fixture(Arc::new(|_, _, _| Ok(())), None);
    let envelope = f.state.handle(f.request.clone()).unwrap().unwrap();
    assert_eq!(f.calls.load(Ordering::SeqCst), 1);
    assert!(!envelope.encode_to_vec().windows(32).any(|b| b == [77; 32]));
    for entry in fs::read_dir(&f.state.root).unwrap() {
        let bytes = fs::read(entry.unwrap().path()).unwrap();
        assert!(!bytes.windows(32).any(|b| b == [77; 32]));
        assert!(!String::from_utf8_lossy(&bytes).contains("content_key"));
    }
    assert_eq!(f.state.handle(f.request.clone()), Err(Error::Replayed));
}

#[test]
fn approval_does_not_extend_request_expiration() {
    let f = fixture(
        Arc::new(|_, _, _| {
            thread::sleep(Duration::from_millis(1100));
            Ok(())
        }),
        None,
    );
    let mut request = f.request.clone();
    if let Some(Operation::RequestUnlock(signed)) = &mut request.operation {
        let binding = signed.request.as_mut().unwrap();
        binding.expires_at = now().unwrap() + 1;
        signed.signature = SigningKey::from_bytes(&[8; 32])
            .sign(&signing_bytes(binding))
            .to_bytes()
            .to_vec();
    }
    assert_eq!(f.state.handle(request), Err(Error::Expired));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
}
