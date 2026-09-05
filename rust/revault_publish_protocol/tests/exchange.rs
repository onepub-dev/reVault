use revault_lockbox_api::{ContactKeyPair, OwnerSigningKeyPair};
use revault_publish_protocol::exchange::*;

fn identity(email: &str) -> (Bundle, OwnerSigningKeyPair) {
    let signing = OwnerSigningKeyPair::generate().unwrap();
    (
        Bundle {
            profile: "work".to_owned(),
            email: email.to_owned(),
            generation: 0,
            encryption_key: ContactKeyPair::generate().unwrap().public_key().to_bytes(),
            signing_key: signing.public_key().to_bytes(),
        },
        signing,
    )
}

fn transcript() -> (Offer, Acceptance) {
    let (alice, a_key) = identity("alice@example.test");
    let (bob, b_key) = identity("bob@example.test");
    let mut offer = Offer {
        version: 1,
        id: random_token().unwrap(),
        inviter: alice,
        recipient_email: bob.email.clone(),
        created_ms: 1000,
        expires_ms: 2000,
        signature: vec![],
    };
    offer.sign(&a_key).unwrap();
    let accept = Acceptance::new(&offer, bob, &b_key).unwrap();
    (offer, accept)
}

#[test]
fn complete_transcript_has_one_full_strength_verification() {
    let (offer, acceptance) = transcript();
    offer.validate(1500).unwrap();
    acceptance.validate(&offer).unwrap();
    let code = verification(&offer, &acceptance).unwrap();
    assert_eq!(code.replace('-', "").len(), 64);
    let round_trip: Offer = decode(&encode(&offer).unwrap()).unwrap();
    assert_eq!(code, verification(&round_trip, &acceptance).unwrap());
    assert!(verification_matches(
        &code,
        &code.to_uppercase().replace('-', " ")
    ));
    assert!(!verification_matches(&code, &code[..20]));
    assert!(!verification_matches(&code, "000000"));
    let mut changed = acceptance.clone();
    changed.recipient.encryption_key[32] ^= 1;
    assert_ne!(code, verification(&offer, &changed).unwrap());
    assert!(changed.validate(&offer).is_err());
}

#[test]
fn signatures_bind_both_keys_identities_roles_generation_and_exchange() {
    let (offer, acceptance) = transcript();
    for field in 0..7 {
        let mut changed = offer.clone();
        match field {
            0 => changed.id = random_token().unwrap(),
            1 => changed.inviter.email = "mallory@example.test".to_owned(),
            2 => changed.recipient_email = "mallory@example.test".to_owned(),
            3 => changed.inviter.generation += 1,
            4 => changed.expires_ms += 1,
            5 => changed.inviter.encryption_key[32] ^= 1,
            _ => changed.signature[10] ^= 1,
        }
        assert!(changed.validate(1500).is_err());
        assert!(acceptance.validate(&changed).is_err());
    }
    let mut swapped = acceptance.clone();
    swapped.recipient = offer.inviter.clone();
    assert!(swapped.validate(&offer).is_err());
    assert!(offer.validate(2000).is_err());
}

#[test]
fn detached_signatures_reject_wrong_key_truncation_and_trailing_bytes() {
    let key = OwnerSigningKeyPair::generate().unwrap();
    let signature = key.sign_detached(b"revault/test/v1");
    key.public_key()
        .verify_detached(b"revault/test/v1", &signature)
        .unwrap();
    assert!(key
        .public_key()
        .verify_detached(b"revault/test/v2", &signature)
        .is_err());
    assert!(OwnerSigningKeyPair::generate()
        .unwrap()
        .public_key()
        .verify_detached(b"revault/test/v1", &signature)
        .is_err());
    assert!(key
        .public_key()
        .verify_detached(b"revault/test/v1", &signature[..60])
        .is_err());
    let mut trailing = signature;
    trailing.push(0);
    assert!(key
        .public_key()
        .verify_detached(b"revault/test/v1", &trailing)
        .is_err());
}

#[test]
fn transport_and_decoding_refuse_unsafe_inputs() {
    for url in [
        "http://example.com",
        "https://user:secret@example.com",
        "https://example.com/?token=secret",
        "https://example.com/#secret",
        "file:///tmp/key",
        "https://example.com/another/path",
        "https://example.com/\n",
    ] {
        assert!(ExchangeClient::new(url).is_err(), "{url}");
    }
    assert!(ExchangeClient::new("http://127.0.0.1:8089").is_ok());
    assert!(decode::<Request>(&vec![0; MAX_MESSAGE_BYTES + 1]).is_err());
    assert!(decode::<Request>(br#"{"operation":"Inspect","id":"abc","extra":1}"#).is_err());
}

#[test]
fn archive_authentication_requires_the_exact_expected_hybrid_signer() {
    use revault_lockbox_api::{Lockbox, LockboxPath, LockboxProtection};
    let signer = OwnerSigningKeyPair::generate().unwrap();
    let recipient = ContactKeyPair::generate().unwrap();
    let mut archive = Lockbox::create_in_memory(
        LockboxProtection::ContactPublicKey {
            name: None,
            contact: recipient.public_key(),
        },
        &signer,
    )
    .unwrap();
    archive
        .add_file(
            &LockboxPath::new("/message").unwrap(),
            b"signed content",
            false,
        )
        .unwrap();
    archive.commit().unwrap();
    assert!(archive
        .owner_signing_public_key_matches(&signer.public_key())
        .unwrap());
    let other = OwnerSigningKeyPair::generate().unwrap();
    assert!(!archive
        .owner_signing_public_key_matches(&other.public_key())
        .unwrap());
}
