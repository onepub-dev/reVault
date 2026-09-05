use revault_lockbox_api::{ContactKeyPair, OwnerSigningKeyPair, SecretString};
use revault_publish_protocol::exchange::*;
use revault_vault_api::VaultDirectory;

#[test]
fn pinned_transcript_and_atomic_verified_bundle_survive_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let password = SecretString::try_from_bytes(b"exchange-vault-test-password".to_vec()).unwrap();
    let alice = OwnerSigningKeyPair::generate().unwrap();
    let bob = OwnerSigningKeyPair::generate().unwrap();
    let identity = |email: &str, signing: &OwnerSigningKeyPair| Bundle {
        profile: "default".to_owned(),
        email: email.to_owned(),
        generation: 0,
        encryption_key: ContactKeyPair::generate().unwrap().public_key().to_bytes(),
        signing_key: signing.public_key().to_bytes(),
    };
    let mut offer = Offer {
        version: 1,
        id: random_token().unwrap(),
        inviter: identity("alice@example.test", &alice),
        recipient_email: "bob@example.test".to_owned(),
        created_ms: 1000,
        expires_ms: 2000,
        signature: vec![],
    };
    offer.sign(&alice).unwrap();
    let acceptance = Acceptance::new(&offer, identity("bob@example.test", &bob), &bob).unwrap();
    let id = offer.id.clone();
    let code = verification(&offer, &acceptance).unwrap();
    let mut local = LocalExchange {
        server: "https://example.test".to_owned(),
        inviter: true,
        token: random_token().unwrap(),
        offer,
        acceptance: None,
        verified_contact: None,
    };
    let vault = VaultDirectory::open_or_create(directory.path(), &password).unwrap();
    vault.save_exchange(&local).unwrap();
    assert!(!vault.contact_exists("bob").unwrap());
    local.acceptance = Some(acceptance.clone());
    vault.save_exchange(&local).unwrap();
    let mut changed = local.clone();
    changed.server = "https://attacker.example".to_owned();
    assert!(vault.save_exchange(&changed).is_err());
    changed = local.clone();
    changed.acceptance = None;
    assert!(vault.save_exchange(&changed).is_err());
    assert!(vault.verify_exchange(&id, "bob", "123456").is_err());
    assert!(!vault.contact_exists("bob").unwrap());
    vault.verify_exchange(&id, "bob", &code).unwrap();
    drop(vault);
    let vault = VaultDirectory::open_or_create(directory.path(), &password).unwrap();
    assert_eq!(
        vault.load_contact("bob").unwrap().to_bytes(),
        acceptance.recipient.encryption_key
    );
    assert_eq!(
        vault.load_contact_signing_key("bob").unwrap().to_bytes(),
        acceptance.recipient.signing_key
    );
    assert_eq!(
        vault
            .load_exchange(&id)
            .unwrap()
            .verified_contact
            .as_deref(),
        Some("bob")
    );
    assert!(vault.save_exchange(&local).is_err()); // Cannot reset the local trust decision.
    vault.verify_exchange(&id, "bob", &code).unwrap();
    vault.forget_exchange(&id).unwrap();
    assert!(vault.list_exchanges().unwrap().is_empty());
    assert!(vault.contact_exists("bob").unwrap());
}
