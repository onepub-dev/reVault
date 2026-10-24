use revault_key_server::exchange_store::{ExchangeLimits, ExchangeStore};
use revault_lockbox_api::{ContactKeyPair, OwnerSigningKeyPair};
use revault_publish_protocol::exchange::*;

fn offer() -> (Offer, Acceptance) {
    let alice = OwnerSigningKeyPair::generate().unwrap();
    let bob = OwnerSigningKeyPair::generate().unwrap();
    let bundle = |email: &str, key: &OwnerSigningKeyPair| Bundle {
        profile: "default".to_owned(),
        email: email.to_owned(),
        generation: 0,
        encryption_key: ContactKeyPair::generate().unwrap().public_key().to_bytes(),
        signing_key: key.public_key().to_bytes(),
    };
    let mut offer = Offer {
        version: 1,
        id: random_token().unwrap(),
        inviter: bundle("alice@example.test", &alice),
        recipient_email: "bob@example.test".to_owned(),
        created_ms: 1000,
        expires_ms: 2000,
        signature: Vec::new(),
    };
    offer.sign(&alice).unwrap();
    let response = Acceptance::new(&offer, bundle("bob@example.test", &bob), &bob).unwrap();
    (offer, response)
}

#[test]
fn durable_reciprocal_lifecycle_capabilities_retries_and_expiry() {
    let dir = tempfile::tempdir().unwrap();
    let store = ExchangeStore::open(dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let (offer, acceptance) = offer();
    let id = offer.id.clone();
    let owner = random_token().unwrap();
    let recipient = random_token().unwrap();
    let create = Request::Create {
        offer: offer.clone(),
        owner_token: owner.clone(),
    };
    assert!(store.handle(create.clone(), 1000).error.is_none());
    assert!(store.handle(create, 1000).error.is_none());
    assert!(store
        .handle(
            Request::Poll {
                id: id.clone(),
                token: id.clone()
            },
            1000
        )
        .error
        .is_some());
    let accept = Request::Accept {
        id: id.clone(),
        recipient_token: recipient.clone(),
        acceptance: acceptance.clone(),
    };
    assert!(store.handle(accept.clone(), 1000).error.is_none());
    assert!(store.handle(accept.clone(), 1000).error.is_none());
    assert!(store
        .handle(Request::Inspect { id: id.clone() }, 1000)
        .acceptance
        .is_none());
    assert!(store
        .handle(
            Request::Accept {
                id: id.clone(),
                recipient_token: random_token().unwrap(),
                acceptance: acceptance.clone()
            },
            1000
        )
        .error
        .is_some());
    assert!(store
        .handle(
            Request::Cancel {
                id: id.clone(),
                owner_token: owner.clone()
            },
            1000
        )
        .error
        .is_some());
    drop(store);
    let store = ExchangeStore::open(dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let received = store.handle(
        Request::Poll {
            id: id.clone(),
            token: owner.clone(),
        },
        1000,
    );
    assert_eq!(received.offer, Some(offer));
    assert_eq!(received.acceptance, Some(acceptance));
    assert!(
        !store
            .handle(
                Request::Complete {
                    id: id.clone(),
                    token: owner
                },
                1000
            )
            .complete
    );
    let complete = Request::Complete {
        id: id.clone(),
        token: recipient,
    };
    assert!(store.handle(complete.clone(), 1000).complete);
    let persisted = std::fs::read_to_string(dir.path().join(format!("{id}.json"))).unwrap();
    assert!(!persisted.contains("alice@example.test"));
    assert!(!persisted.contains("bob@example.test"));
    assert!(!persisted.contains("encryption_key"));
    assert!(!persisted.contains("signing_key"));
    drop(store);
    let store = ExchangeStore::open(dir.path(), ExchangeLimits::default(), 1000).unwrap();
    assert!(store.handle(complete, 1000).complete);
    assert!(store.handle(accept.clone(), 1000).complete);
    assert!(store.handle(accept, 2000).error.is_some());
    assert!(!dir.path().join(format!("{id}.json")).exists());
}

#[test]
fn capacity_refuses_new_work_without_evicting_an_admitted_exchange() {
    let dir = tempfile::tempdir().unwrap();
    let limits = ExchangeLimits {
        invitations: 1,
        bytes: MAX_MESSAGE_BYTES,
        per_identity: 1,
    };
    let store = ExchangeStore::open(dir.path(), limits, 1000).unwrap();
    let (first, acceptance) = offer();
    let id = first.id.clone();
    let owner_token = random_token().unwrap();
    assert!(store
        .handle(
            Request::Create {
                offer: first,
                owner_token: owner_token.clone()
            },
            1000
        )
        .error
        .is_none());
    assert!(store
        .handle(
            Request::Create {
                offer: offer().0,
                owner_token: random_token().unwrap()
            },
            1000
        )
        .error
        .is_some());
    assert!(store
        .handle(
            Request::Accept {
                id: id.clone(),
                recipient_token: random_token().unwrap(),
                acceptance
            },
            1000
        )
        .error
        .is_none());
    assert!(store
        .handle(
            Request::Poll {
                id,
                token: owner_token
            },
            1000
        )
        .acceptance
        .is_some());
}

#[test]
fn replica_restart_preserves_acceptance_and_terminal_receipts() {
    let primary_dir = tempfile::tempdir().unwrap();
    let replica_dir = tempfile::tempdir().unwrap();
    let primary = ExchangeStore::open(primary_dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let replica = ExchangeStore::open(replica_dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let (offer, acceptance) = offer();
    let id = offer.id.clone();
    let owner = random_token().unwrap();
    let recipient = random_token().unwrap();
    assert!(primary
        .handle(
            Request::Create {
                offer,
                owner_token: owner.clone()
            },
            1000
        )
        .error
        .is_none());
    let initial = primary.snapshot(&id).unwrap();
    replica.apply_snapshot(&initial, 1000).unwrap();
    assert!(primary
        .handle(
            Request::Accept {
                id: id.clone(),
                acceptance: acceptance.clone(),
                recipient_token: recipient.clone()
            },
            1000
        )
        .error
        .is_none());
    let accepted = primary.snapshot(&id).unwrap();
    replica.apply_snapshot(&accepted, 1000).unwrap();
    replica.apply_snapshot(&initial, 1000).unwrap();
    assert_eq!(
        replica
            .handle(
                Request::Poll {
                    id: id.clone(),
                    token: owner.clone()
                },
                1000
            )
            .acceptance,
        Some(acceptance)
    );
    primary.handle(
        Request::Complete {
            id: id.clone(),
            token: owner.clone(),
        },
        1000,
    );
    assert!(
        primary
            .handle(
                Request::Complete {
                    id: id.clone(),
                    token: recipient.clone()
                },
                1000
            )
            .complete
    );
    replica
        .apply_snapshot(&primary.snapshot(&id).unwrap(), 1000)
        .unwrap();
    replica.apply_snapshot(&accepted, 1000).unwrap();
    drop(replica);
    let replica = ExchangeStore::open(replica_dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let response = replica.handle(
        Request::Complete {
            id,
            token: recipient,
        },
        1000,
    );
    assert!(response.complete);
    assert!(response.offer.is_none());
    assert!(response.acceptance.is_none());
}

#[test]
fn acknowledged_exchange_survives_explicit_replica_promotion() {
    use revault_key_server::store::{PublishStore, ServerConfig};
    use std::sync::Arc;
    let primary_dir = tempfile::tempdir().unwrap();
    let replica_dir = tempfile::tempdir().unwrap();
    let replica_config = ServerConfig {
        state_dir: replica_dir.path().to_owned(),
        server_id: 1,
        replication_token: Some("synthetic-replication-token".to_owned()),
        ..Default::default()
    };
    let replica = Arc::new(PublishStore::open(replica_config.clone()).unwrap());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let served = Arc::clone(&replica);
    std::thread::spawn(move || revault_key_server::server::run_listener(listener, served).unwrap());
    let primary = PublishStore::open(ServerConfig {
        state_dir: primary_dir.path().to_owned(),
        replication_token: replica_config.replication_token.clone(),
        replication_peer_urls: vec![format!("http://{address}/v1/replicate")],
        ..Default::default()
    })
    .unwrap();
    // Protocol fixtures use short synthetic timestamps. Replication validates
    // wall time, so sign a fresh offer for this real HTTP lifecycle.
    let key = OwnerSigningKeyPair::generate().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut offer = offer().0;
    offer.created_ms = now;
    offer.expires_ms = now + 60_000;
    offer.inviter.signing_key = key.public_key().to_bytes();
    offer.sign(&key).unwrap();
    let id = offer.id.clone();
    let owner_token = random_token().unwrap();
    let create = Request::Create {
        offer: offer.clone(),
        owner_token: owner_token.clone(),
    };
    assert!(
        replica.handle_exchange(create.clone(), now).error.is_some(),
        "standby must not create"
    );
    let created = primary.handle_exchange(create.clone(), now);
    assert!(created.error.is_none(), "{:?}", created.error);
    drop(primary);
    // Use a copied persisted state directory to model a stopped/restarted
    // standby without opening its live storage twice in this test process.
    let promoted_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(promoted_dir.path().join("exchanges")).unwrap();
    std::fs::copy(
        replica_dir.path().join(format!("exchanges/{id}.json")),
        promoted_dir.path().join(format!("exchanges/{id}.json")),
    )
    .unwrap();
    let promoted = PublishStore::open(ServerConfig {
        state_dir: promoted_dir.path().to_owned(),
        server_id: 1,
        promoted_owner_ids: vec![0],
        ..Default::default()
    })
    .unwrap();
    let received = promoted.handle_exchange(
        Request::Poll {
            id,
            token: owner_token,
        },
        now,
    );
    assert!(received.error.is_none());
    assert_eq!(received.offer, Some(offer));
}

#[test]
fn cancellation_requires_owner_and_removes_only_unaccepted_invitation() {
    let dir = tempfile::tempdir().unwrap();
    let store = ExchangeStore::open(dir.path(), ExchangeLimits::default(), 1000).unwrap();
    let (offer, _) = offer();
    let id = offer.id.clone();
    let owner_token = random_token().unwrap();
    store.handle(
        Request::Create {
            offer,
            owner_token: owner_token.clone(),
        },
        1000,
    );
    assert!(store
        .handle(
            Request::Cancel {
                id: id.clone(),
                owner_token: random_token().unwrap()
            },
            1000
        )
        .error
        .is_some());
    assert!(store
        .handle(
            Request::Cancel {
                id: id.clone(),
                owner_token: owner_token.clone()
            },
            1000
        )
        .error
        .is_none());
    assert!(store
        .handle(Request::Inspect { id: id.clone() }, 1000)
        .error
        .is_some());
    drop(store);
    let store = ExchangeStore::open(dir.path(), ExchangeLimits::default(), 1000).unwrap();
    assert!(
        store
            .handle(Request::Cancel { id, owner_token }, 1000)
            .complete
    );
}
