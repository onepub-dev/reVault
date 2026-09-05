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
    assert!(store.handle(complete, 1000).complete);
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
                owner_token
            },
            1000
        )
        .error
        .is_none());
    assert!(store.handle(Request::Inspect { id }, 1000).error.is_some());
}
