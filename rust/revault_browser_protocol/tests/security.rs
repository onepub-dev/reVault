use ed25519_dalek::{Signer, SigningKey};
use revault_browser_protocol::{
    crypto::{seal, RecipientKey},
    receiver::{Application, Receiver},
    transport,
    validation::{signing_bytes, validate_origin, validate_request, verify_signed},
    *,
};
use revault_lockbox_api::SecretVec;
use std::io::Cursor;

fn fixture() -> (SigningKey, SignedUnlockRequest, RecipientKey) {
    let signing = SigningKey::from_bytes(&[9; 32]);
    let recipient = RecipientKey::generate().unwrap();
    let request = UnlockRequest {
        protocol_version: 1,
        request_id: "request-1".into(),
        application_id: "test".into(),
        server_boot_id: vec![1; 32],
        challenge: vec![2; 32],
        issued_at: 100,
        expires_at: 200,
        recipient_key_id: "boot-1".into(),
        recipient_public_key: recipient.public_key().into(),
        lockbox_id: "lockbox-1".into(),
        requested_scope: SCOPE.into(),
        origin: "https://test.example".into(),
    };
    let signature = signing.sign(&signing_bytes(&request)).to_bytes().to_vec();
    (
        signing,
        SignedUnlockRequest {
            request: Some(request),
            signature,
        },
        recipient,
    )
}
#[test]
fn signatures_reject_every_substitution() {
    let (signing, signed, _) = fixture();
    assert!(verify_signed(&signed, &signing.verifying_key().to_bytes(), 150).is_ok());
    let modifications: Vec<fn(&mut UnlockRequest)> = vec![
        |r| r.protocol_version += 1,
        |r| r.request_id.push('x'),
        |r| r.application_id.push('x'),
        |r| r.server_boot_id[0] ^= 1,
        |r| r.challenge[0] ^= 1,
        |r| r.issued_at += 1,
        |r| r.expires_at += 1,
        |r| r.recipient_key_id.push('x'),
        |r| r.recipient_public_key[0] ^= 1,
        |r| r.lockbox_id.push('x'),
        |r| r.requested_scope.push('x'),
        |r| r.origin.push('x'),
    ];
    for modify in modifications {
        let mut bad = signed.clone();
        modify(bad.request.as_mut().unwrap());
        assert!(verify_signed(&bad, &signing.verifying_key().to_bytes(), 150).is_err());
    }
    assert!(verify_signed(
        &signed,
        &SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes(),
        150
    )
    .is_err());
}
#[test]
fn hpke_rejects_tampering_and_wrong_recipient() {
    let (_, signed, recipient) = fixture();
    let request = signed.request.unwrap();
    let key = SecretVec::try_from_slice(&[77; 32]).unwrap();
    let envelope = seal(&key, &request).unwrap();
    assert!(recipient.open(&envelope, &request, 150).is_ok());
    assert!(!envelope.encode_to_vec().windows(32).any(|s| s == [77; 32]));
    for part in 0..4 {
        let mut bad = envelope.clone();
        match part {
            0 => bad.ciphertext[0] ^= 1,
            1 => bad.tag[0] ^= 1,
            2 => bad.encapsulated_key[0] ^= 1,
            _ => bad.binding.as_mut().unwrap().challenge[0] ^= 1,
        }
        assert!(recipient.open(&bad, &request, 150).is_err());
    }
    // Even changing the caller's expected metadata cannot defeat HPKE AAD.
    let mut changed = request.clone();
    changed.origin = "https://other.example".into();
    let mut bad = envelope.clone();
    bad.binding = Some(changed.clone());
    assert!(recipient.open(&bad, &changed, 150).is_err());
    assert!(RecipientKey::generate()
        .unwrap()
        .open(&envelope, &request, 150)
        .is_err());
    assert!(recipient.open(&envelope, &request, 200).is_err());
}
#[test]
fn strict_origins_and_expiry() {
    for origin in [
        "http://test.example",
        "https://test.example/",
        "https://user@test.example",
        "https://test.example:443",
        "null",
        "https://test.example/a",
        "https://test.example#x",
    ] {
        assert!(validate_origin(origin).is_err(), "{origin}");
    }
    assert!(validate_origin("https://test.example:8443").is_ok());
    let (_, signed, _) = fixture();
    let mut r = signed.request.unwrap();
    assert_eq!(validate_request(&r, 99), Err(Error::Expired));
    assert_eq!(validate_request(&r, 200), Err(Error::Expired));
    r.expires_at = 221;
    assert_eq!(validate_request(&r, 150), Err(Error::Expired));
}
#[test]
fn native_transport_bounds_and_partial_frames() {
    let message = response(Err(Error::VaultLocked));
    let mut bytes = Vec::new();
    transport::write_frame(&mut bytes, &message).unwrap();
    let decoded = transport::read_frame(&mut Cursor::new(&bytes))
        .unwrap()
        .unwrap();
    assert_eq!(
        BrowserResponse::decode(decoded.as_slice()).unwrap(),
        message
    );
    assert!(transport::read_frame(&mut Cursor::new([]))
        .unwrap()
        .is_none());
    for len in 1..bytes.len() {
        assert!(transport::read_frame(&mut Cursor::new(&bytes[..len])).is_err());
    }
    for len in [0, transport::MAX_JSON_BYTES as u32 + 1, u32::MAX] {
        assert!(transport::read_frame(&mut Cursor::new(len.to_ne_bytes())).is_err());
    }
    for json in [
        r#"{"payload":"AA==","path":"/secret"}"#,
        r#"{"payload":"?"}"#,
        r#"{"payload":"AA==","payload":"AQ=="}"#,
        r#"[]"#,
    ] {
        assert!(transport::decode_json(json.as_bytes()).is_err());
    }
    let mut concatenated = bytes.clone();
    concatenated.extend(&bytes);
    let mut input = Cursor::new(concatenated);
    assert!(transport::read_frame(&mut input).unwrap().is_some());
    assert!(transport::read_frame(&mut input).unwrap().is_some());
    assert!(transport::read_frame(&mut input).unwrap().is_none());
}
#[test]
fn receiver_sessions_replay_restart_and_new_approval() {
    let signing = SigningKey::from_bytes(&[9; 32]);
    let app = Application {
        origin: "https://test.example".into(),
        application_id: "test".into(),
        lockbox_id: "lockbox-1".into(),
    };
    let key = SecretVec::try_from_slice(&[77; 32]).unwrap();
    let mut receiver = Receiver::new().unwrap();
    let signed = receiver
        .issue(&app, [5; 32], 100, |bytes| {
            Ok(signing.sign(bytes).to_bytes())
        })
        .unwrap();
    let envelope = seal(&key, signed.request.as_ref().unwrap()).unwrap();
    assert!(matches!(
        receiver.accept(&[6; 32], &envelope, 150),
        Err(Error::Denied)
    ));
    assert!(receiver.accept(&[5; 32], &envelope, 150).is_ok());
    assert!(matches!(
        receiver.accept(&[5; 32], &envelope, 150),
        Err(Error::Replayed)
    ));
    drop(receiver);
    let mut receiver = Receiver::new().unwrap();
    assert!(receiver.accept(&[5; 32], &envelope, 150).is_err());
    let signed = receiver
        .issue(&app, [5; 32], 150, |bytes| {
            Ok(signing.sign(bytes).to_bytes())
        })
        .unwrap();
    let fresh = seal(&key, signed.request.as_ref().unwrap()).unwrap();
    assert!(receiver.accept(&[5; 32], &fresh, 160).is_ok());
}
#[test]
fn malformed_input_smoke_fuzz() {
    // Deterministic malformed-input corpus. libFuzzer target adds coverage guidance.
    let mut state = 13u64;
    for length in 0..1024 {
        let mut data = vec![0u8; length];
        for byte in &mut data {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        let _ = transport::decode_json(&data);
        let _ = transport::read_frame(&mut Cursor::new(&data));
        let _ = BrowserRequest::decode(data.as_slice());
    }
}

#[test]
fn hpke_dependency_retains_zeroize_on_drop() {
    fn requires_zeroize<T: zeroize::Zeroize>() {}
    requires_zeroize::<x25519_dalek::StaticSecret>();
    requires_zeroize::<x25519_dalek::SharedSecret>();
    assert!(std::mem::needs_drop::<x25519_dalek::StaticSecret>());
    assert!(std::mem::needs_drop::<x25519_dalek::SharedSecret>());
}
