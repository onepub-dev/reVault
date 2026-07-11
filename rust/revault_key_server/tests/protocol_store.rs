use std::fs;
use std::io::ErrorKind;
use std::net::IpAddr;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use lockbox_key_server::store::{PublishStore, ServerConfig, SmtpTlsMode, StoreError};
use lockbox_publish_protocol::client::{ContactPublish, PublishClient};
use lockbox_publish_protocol::protocol::{
    decode_request, encode_delete_request, encode_publish_request, encode_receive_request,
    Operation, Reader, Status,
};
use lockbox_publish_protocol::{
    encode_replication_request, sign_replication_event, ClusterTopology, ReplicationEvent,
    ReplicationEventKind, ReplicationRequest, ServerStatus, TopologyRoute, TopologyServer,
};

fn contact_payload(label: &str) -> Vec<u8> {
    lockbox_publish_protocol::payload::encode_contact_publish(
        &format!("{label}@example.com"),
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    )
}

fn key_replacement_payload(label: &str) -> Vec<u8> {
    lockbox_publish_protocol::payload::encode_key_replacement(
        lockbox_publish_protocol::payload::KeyReplacement {
            identity: &format!("{label}@example.com"),
            old_fingerprint: &[3_u8; 32],
            new_public_key: b"replacement-public-key-material",
            new_signing_public_key: b"replacement-signing-public-key-material",
            new_fingerprint: &[4_u8; 32],
            replacement_nonce: &[5_u8; 24],
            signature_by_old_key: b"signature-by-old-key",
            created_at_unix_ms: 1,
            expires_at_unix_ms: 2,
        },
    )
}

#[test]
fn protocol_round_trips_publish_request() {
    let request = encode_publish_request(900, 3, b"candidate");
    let decoded = decode_request(&request, 1024).unwrap();
    assert_eq!(decoded.operation, Operation::Publish);

    let mut reader = Reader::new(&decoded.payload);
    reader.message_version().unwrap();
    assert_eq!(reader.u32().unwrap(), 900);
    assert_eq!(reader.u16().unwrap(), 3);
    assert_eq!(reader.bytes().unwrap(), b"candidate");
}

#[test]
fn store_creates_receives_and_deletes_publish() {
    let (_guard, config) = temp_config("flow");
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("flow");

    let create = create_publish(&store, 900, 2, &payload, "flow@example.test");
    verify_publish(&store, &create);

    let received = store.receive(&create.publish_code).unwrap();
    assert_eq!(received.payload, payload);
    assert_eq!(received.remaining_receives, 1);

    assert!(store
        .delete(&create.publish_code, &create.delete_token)
        .unwrap());
    assert!(store.receive(&create.publish_code).is_err());
}

#[test]
fn store_attaches_email_verification_to_pending_publish() {
    let (_guard, mut config) = temp_config("email-verification");
    config.public_url = Some("https://publish.example.test/v1/publish".to_string());
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("email-verification");
    let create = store
        .create_from_payload(
            &decode_request(
                &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
                    0,
                    2,
                    &payload,
                    Some("alice@example.test"),
                ),
                2048,
            )
            .unwrap()
            .payload,
        )
        .unwrap();

    let verification_url = create.verification_url.as_deref().unwrap();
    assert!(verification_url.starts_with("https://publish.example.test/v1/verify?"));
    assert!(matches!(
        store.receive(&create.publish_code),
        Err(StoreError::EmailUnverified)
    ));

    let (code, token) = verification_query_parts(verification_url);
    assert_eq!(code, create.publish_code);
    let page = store.verify_email(&code, &token);
    assert!(page.success);

    let second_receive = store.receive(&create.publish_code).unwrap();
    let second_verification = second_receive.email_verification.unwrap();
    assert_eq!(second_verification.email, "alice@example.test");
    assert!(second_verification.verified);
    assert!(second_verification.verified_at_unix_ms > 0);
    assert!(!second_verification.attestation.is_empty());
    assert!(second_receive.expires_at_ms >= unix_ms_now() + 7_000_000);
}

#[test]
fn store_does_not_receive_verified_publish_by_publisher_email_without_publish_code() {
    let (_guard, mut config) = temp_config("email-receive");
    config.public_url = Some("https://publish.example.test/v1/publish".to_string());
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("email-receive");
    let request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &payload,
            Some("publisher@example.test"),
        ),
        2048,
    )
    .unwrap();
    let create = store.create_from_payload(&request.payload).unwrap();

    let pending_receive =
        decode_request(&encode_receive_request("publisher@example.test"), 2048).unwrap();
    assert_status(
        &store.handle(pending_receive.operation, &pending_receive.payload),
        Status::PublishNotFound,
    );

    let (code, token) = verification_query_parts(create.verification_url.as_deref().unwrap());
    assert_eq!(code, create.publish_code);
    assert!(store.verify_email(&code, &token).success);

    let verified_receive =
        decode_request(&encode_receive_request("publisher@example.test"), 2048).unwrap();
    let response = store.handle(verified_receive.operation, &verified_receive.payload);
    assert_status(&response, Status::PublishNotFound);

    let received = store.receive(&create.publish_code).unwrap();
    assert_eq!(received.payload, payload);
}

#[test]
fn store_rate_limits_verification_email_by_contact() {
    let (_guard, mut config) = temp_config("email-rate-contact");
    config.verification_email_rate_limit_per_hour = 1;
    config.verification_email_ip_rate_limit_per_hour = 0;
    let store = PublishStore::open(config).unwrap();

    let first = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-rate-contact-a"),
            Some("alice@example.test"),
        ),
        2048,
    )
    .unwrap();
    store.create_from_payload(&first.payload).unwrap();

    let second = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-rate-contact-b"),
            Some("ALICE@example.test"),
        ),
        2048,
    )
    .unwrap();
    assert!(matches!(
        store.create_from_payload(&second.payload),
        Err(StoreError::RateLimited)
    ));
    assert!(store.is_verification_email_blocked("alice@example.test"));

    let malformed = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            b"not-a-contact-payload",
            Some("alice@example.test"),
        ),
        2048,
    )
    .unwrap();
    assert!(matches!(
        store.create_from_payload(&malformed.payload),
        Err(StoreError::RateLimited)
    ));
}

#[test]
fn store_rejects_verification_email_publish_on_non_owner_server() {
    let (_guard_0, mut config_0) = temp_config("email-owner-0");
    let (_guard_1, mut config_1) = temp_config("email-owner-1");
    let (_guard_2, mut config_2) = temp_config("email-owner-2");
    let servers = vec![
        TopologyServer {
            id: 0,
            url: "http://publish0.example.test/v1/publish".to_string(),
            status: ServerStatus::Active,
            last_seen_ms: None,
        },
        TopologyServer {
            id: 1,
            url: "http://publish1.example.test/v1/publish".to_string(),
            status: ServerStatus::Active,
            last_seen_ms: None,
        },
        TopologyServer {
            id: 2,
            url: "http://publish2.example.test/v1/publish".to_string(),
            status: ServerStatus::Active,
            last_seen_ms: None,
        },
    ];
    let routes = vec![
        TopologyRoute {
            owner_id: 0,
            primary_id: 0,
            failover_ids: vec![1],
        },
        TopologyRoute {
            owner_id: 1,
            primary_id: 1,
            failover_ids: vec![2],
        },
        TopologyRoute {
            owner_id: 2,
            primary_id: 2,
            failover_ids: vec![0],
        },
    ];

    for config in [&mut config_0, &mut config_1, &mut config_2] {
        config.cluster_id = "email-owner-cluster".to_string();
        config.topology_servers = servers.clone();
        config.topology_routes = routes.clone();
        config.verification_email_rate_limit_per_hour = 0;
        config.verification_email_ip_rate_limit_per_hour = 0;
    }
    config_0.server_id = 0;
    config_1.server_id = 1;
    config_2.server_id = 2;

    let topology = ClusterTopology {
        cluster_id: "email-owner-cluster".to_string(),
        version: 1,
        servers,
        routes,
    };
    let allowed_ids = topology.verification_email_server_ids("victim@example.test");
    assert_eq!(allowed_ids.len(), 2);
    let disallowed_id = [0_u8, 1, 2]
        .into_iter()
        .find(|server_id| !allowed_ids.contains(server_id))
        .unwrap();
    let store_0 = PublishStore::open(config_0).unwrap();
    let store_1 = PublishStore::open(config_1).unwrap();
    let store_2 = PublishStore::open(config_2).unwrap();
    let allowed_store = match allowed_ids[0] {
        0 => &store_0,
        1 => &store_1,
        _ => &store_2,
    };
    let backup_store = match allowed_ids[1] {
        0 => &store_0,
        1 => &store_1,
        _ => &store_2,
    };
    let disallowed_store = match disallowed_id {
        0 => &store_0,
        1 => &store_1,
        _ => &store_2,
    };

    let disallowed_request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-owner-non-owner"),
            Some("Victim@Example.Test"),
        ),
        2048,
    )
    .unwrap();
    assert!(matches!(
        disallowed_store.create_from_payload(&disallowed_request.payload),
        Err(StoreError::RateLimited)
    ));

    let allowed_request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-owner-owner"),
            Some("Victim@Example.Test"),
        ),
        2048,
    )
    .unwrap();
    allowed_store
        .create_from_payload(&allowed_request.payload)
        .unwrap();

    let backup_request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-owner-backup"),
            Some("Victim@Example.Test"),
        ),
        2048,
    )
    .unwrap();
    backup_store
        .create_from_payload(&backup_request.payload)
        .unwrap();
}

#[test]
fn store_rate_limits_verification_email_by_ip_address() {
    let (_guard, mut config) = temp_config("email-rate-ip");
    config.verification_email_rate_limit_per_hour = 0;
    config.verification_email_ip_rate_limit_per_hour = 1;
    let store = PublishStore::open(config).unwrap();
    let peer_ip = Some(IpAddr::from([127, 0, 0, 1]));

    let first = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-rate-ip-a"),
            Some("alice@example.test"),
        ),
        2048,
    )
    .unwrap();
    store
        .create_from_payload_with_peer(&first.payload, peer_ip)
        .unwrap();

    let second = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-rate-ip-b"),
            Some("bob@example.test"),
        ),
        2048,
    )
    .unwrap();
    let response = store.handle_with_peer(second.operation, &second.payload, peer_ip);
    assert_status(&response, Status::RateLimited);
    assert!(store.is_rate_limit_blocked(peer_ip));

    let malformed = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            b"not-a-contact-payload",
            Some("charlie@example.test"),
        ),
        2048,
    )
    .unwrap();
    let response = store.handle_with_peer(malformed.operation, &malformed.payload, peer_ip);
    assert_status(&response, Status::RateLimited);
}

#[test]
fn store_reports_clean_email_send_failure_when_smtp_is_not_configured() {
    let (_guard, mut config) = temp_config("email-send-not-configured");
    config.developer_mode = false;
    config.smtp_host = None;
    let store = PublishStore::open(config).unwrap();
    let request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &contact_payload("email-send-not-configured"),
            Some("alice@example.test"),
        ),
        2048,
    )
    .unwrap();

    let response = store.handle(request.operation, &request.payload);
    assert_status(&response, Status::StoreUnavailable);
    let (_, message) = lockbox_publish_protocol::protocol::decode_error_payload(&response[14..])
        .expect("error payload");
    assert_eq!(message, "could not send verification email");
}

#[test]
fn store_enforces_receive_limit() {
    let (_guard, config) = temp_config("receive-limit");
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("receive-limit");
    let create = create_publish(&store, 900, 1, &payload, "receive-limit@example.test");
    verify_publish(&store, &create);

    assert!(store.receive(&create.publish_code).is_ok());
    assert!(store.receive(&create.publish_code).is_err());
}

fn verification_query_parts(url: &str) -> (String, String) {
    let query = url.split_once('?').unwrap().1;
    let mut code = None;
    let mut token = None;
    for part in query.split('&') {
        let (key, value) = part.split_once('=').unwrap();
        match key {
            "code" => code = Some(value.to_string()),
            "token" => token = Some(value.to_string()),
            _ => {}
        }
    }
    (code.unwrap(), token.unwrap())
}

fn create_publish(
    store: &PublishStore,
    ttl_seconds: u32,
    max_receives: u16,
    payload: &[u8],
    email: &str,
) -> lockbox_key_server::store::CreatedPublish {
    store
        .create_from_payload(
            &decode_request(
                &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
                    ttl_seconds,
                    max_receives,
                    payload,
                    Some(email),
                ),
                2048,
            )
            .unwrap()
            .payload,
        )
        .unwrap()
}

fn verify_publish(store: &PublishStore, create: &lockbox_key_server::store::CreatedPublish) {
    let (code, token) = verification_query_parts(create.verification_url.as_deref().unwrap());
    assert_eq!(code, create.publish_code);
    assert!(store.verify_email(&code, &token).success);
}

#[test]
fn publish_codes_use_fixed_digit_count() {
    let (_guard, mut config) = temp_config("code-digits");
    config.server_id = 7;
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("code-digits");
    let create = create_publish(&store, 900, 1, &payload, "code-digits@example.test");

    assert_eq!(create.publish_code.len(), 14);
    assert!(create.publish_code.starts_with("77"));
    assert!(create
        .publish_code
        .chars()
        .skip(2)
        .all(|character| character.is_ascii_digit()));
}

#[test]
fn store_rejects_invalid_server_id() {
    let (_guard, mut config) = temp_config("bad-server-id");
    config.server_id = 36;
    match PublishStore::open(config) {
        Ok(_) => panic!("invalid server id should be rejected"),
        Err(err) => assert!(err.to_string().contains("server id")),
    }
}

#[test]
fn store_publishes_configured_topology() {
    let (_guard, mut config) = temp_config("topology");
    config.cluster_id = "acme".to_string();
    config.topology_version = 7;
    config.topology_servers = vec![
        TopologyServer {
            id: 0,
            url: "http://publish0.example/v1/publish".to_string(),
            status: ServerStatus::Active,
            last_seen_ms: None,
        },
        TopologyServer {
            id: 1,
            url: "http://publish1.example/v1/publish".to_string(),
            status: ServerStatus::Standby,
            last_seen_ms: None,
        },
    ];
    config.topology_routes = vec![TopologyRoute {
        owner_id: 0,
        primary_id: 0,
        failover_ids: vec![1],
    }];
    let store = PublishStore::open(config).unwrap();
    let topology = store.topology();

    assert_eq!(topology.cluster_id, "acme");
    assert_eq!(topology.version, 7);
    assert_eq!(topology.servers.len(), 2);
    assert_eq!(topology.routes[0].failover_ids, vec![1]);
    lockbox_publish_protocol::encode_topology(&topology).unwrap();
}

#[test]
fn replicated_publish_is_served_only_after_owner_promotion() {
    let (_guard, mut config) = temp_config("replica-promote");
    config.server_id = 1;
    config.replication_token = Some("peer-secret".to_string());
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("replica-promote");
    let event = ReplicationEvent {
        origin_server_id: 0,
        origin_epoch: 1,
        origin_sequence: 1,
        kind: ReplicationEventKind::PutPublish {
            publish_code: "00123456789012".to_string(),
            delete_token_hash: vec![9_u8; 16],
            payload: payload.clone(),
            contact_email: Some("replica-promote@example.com".to_string()),
            expires_at_unix_ms: unix_ms_now() + 900_000,
            receive_ttl_ms: 900_000,
            email_verified_at_unix_ms: unix_ms_now(),
            max_receives: 1,
            receives: 0,
        },
    };
    let request = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    });
    let request = decode_request(&request, 4096).unwrap();
    assert_eq!(request.operation, Operation::Replicate);
    assert!(store.apply_replication_payload(&request.payload).unwrap());

    assert!(store.receive("00123456789012").is_err());

    let (_guard, mut config) = temp_config("replica-promoted");
    config.server_id = 1;
    config.replication_token = Some("peer-secret".to_string());
    config.promoted_owner_ids.push(0);
    let store = PublishStore::open(config).unwrap();
    let event = ReplicationEvent {
        origin_server_id: 0,
        origin_epoch: 1,
        origin_sequence: 1,
        kind: ReplicationEventKind::PutPublish {
            publish_code: "00123456789012".to_string(),
            delete_token_hash: vec![9_u8; 16],
            payload: payload.clone(),
            contact_email: Some("replica-promote@example.com".to_string()),
            expires_at_unix_ms: unix_ms_now() + 900_000,
            receive_ttl_ms: 900_000,
            email_verified_at_unix_ms: unix_ms_now(),
            max_receives: 1,
            receives: 0,
        },
    };
    let request = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    });
    let request = decode_request(&request, 4096).unwrap();
    assert!(store.apply_replication_payload(&request.payload).unwrap());
    assert_eq!(store.receive("00123456789012").unwrap().payload, payload);
}

#[test]
fn replication_sequence_is_idempotent_after_restart() {
    let (_guard, mut config) = temp_config("replica-idempotent");
    config.server_id = 1;
    config.replication_token = Some("peer-secret".to_string());
    config.promoted_owner_ids.push(0);
    let payload = contact_payload("replica-idempotent");
    let event = ReplicationEvent {
        origin_server_id: 0,
        origin_epoch: 2,
        origin_sequence: 7,
        kind: ReplicationEventKind::PutPublish {
            publish_code: "00123456789012".to_string(),
            delete_token_hash: vec![9_u8; 16],
            payload: payload.clone(),
            contact_email: Some("replica-idempotent@example.com".to_string()),
            expires_at_unix_ms: unix_ms_now() + 900_000,
            receive_ttl_ms: 900_000,
            email_verified_at_unix_ms: unix_ms_now(),
            max_receives: 2,
            receives: 0,
        },
    };
    let request = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    });
    let request = decode_request(&request, 4096).unwrap();
    {
        let store = PublishStore::open(config.clone()).unwrap();
        assert!(store.apply_replication_payload(&request.payload).unwrap());
    }
    let store = PublishStore::open(config).unwrap();
    assert!(!store.apply_replication_payload(&request.payload).unwrap());
    assert_eq!(store.receive("00123456789012").unwrap().payload, payload);
}

#[test]
fn replicated_rate_limit_block_marks_client_blocked() {
    let (_guard, mut config) = temp_config("replica-rate-limit-block");
    config.server_id = 1;
    config.replication_token = Some("peer-secret".to_string());
    let store = PublishStore::open(config).unwrap();
    let peer_ip = IpAddr::from([203, 0, 113, 7]);
    let event = ReplicationEvent {
        origin_server_id: 0,
        origin_epoch: 2,
        origin_sequence: 8,
        kind: ReplicationEventKind::RateLimitBlock {
            client_ip: peer_ip.to_string(),
            expires_at_unix_ms: unix_ms_now() + 60_000,
        },
    };
    let request = encode_replication_request(&ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    });
    let request = decode_request(&request, 4096).unwrap();

    assert!(store.apply_replication_payload(&request.payload).unwrap());
    assert!(store.is_rate_limit_blocked(Some(peer_ip)));
    assert!(!store.is_rate_limit_blocked(Some(IpAddr::from([203, 0, 113, 8]))));
}

#[test]
fn store_rejects_untyped_payload_bytes() {
    let (_guard, config) = temp_config("reject-untyped");
    let store = PublishStore::open(config).unwrap();
    let request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            b"random crap",
            Some("reject-untyped@example.test"),
        ),
        2048,
    )
    .unwrap();

    assert!(store.create_from_payload(&request.payload).is_err());

    let response = store.handle(request.operation, &request.payload);
    assert_status(&response, Status::MalformedRequest);
}

#[test]
fn store_accepts_versioned_key_replacement_payload() {
    let (_guard, config) = temp_config("key-replacement");
    let store = PublishStore::open(config).unwrap();
    let payload = key_replacement_payload("key-replacement");
    let create = create_publish(&store, 900, 1, &payload, "key-replacement@example.test");
    verify_publish(&store, &create);

    assert_eq!(
        store.receive(&create.publish_code).unwrap().payload,
        payload
    );
}

#[test]
fn payload_validator_rejects_bad_message_version() {
    let mut payload = contact_payload("bad-version");
    payload[4..6].copy_from_slice(&2_u16.to_be_bytes());

    assert!(lockbox_publish_protocol::payload::validate_payload(&payload).is_err());
}

#[test]
fn delete_request_rejects_bad_message_version() {
    let (_guard, config) = temp_config("bad-delete-version");
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("bad-delete-version");
    let create = create_publish(&store, 900, 1, &payload, "bad-delete-version@example.test");
    let mut request = decode_request(
        &encode_delete_request(&create.publish_code, &create.delete_token),
        1024,
    )
    .unwrap();
    request.payload[0..2].copy_from_slice(&2_u16.to_be_bytes());

    let response = store.handle(request.operation, &request.payload);
    assert_status(&response, Status::MalformedRequest);
}

#[test]
fn store_replays_live_records_from_disk() {
    let (guard, config) = temp_config("replay");
    let payload = contact_payload("replay");
    let publish_code = {
        let store = PublishStore::open(config.clone()).unwrap();
        let create = create_publish(&store, 900, 1, &payload, "replay@example.test");
        verify_publish(&store, &create);
        create.publish_code
    };

    let reopened = PublishStore::open(config).unwrap();
    assert_eq!(reopened.receive(&publish_code).unwrap().payload, payload);
    drop(guard);
}

#[test]
fn store_replays_receive_count_after_restart() {
    let (_guard, config) = temp_config("receive-count-replay");
    let payload = contact_payload("receive-count-replay");
    let publish_code = {
        let store = PublishStore::open(config.clone()).unwrap();
        let create = create_publish(
            &store,
            900,
            2,
            &payload,
            "receive-count-replay@example.test",
        );
        verify_publish(&store, &create);
        assert_eq!(
            store
                .receive(&create.publish_code)
                .unwrap()
                .remaining_receives,
            1
        );
        create.publish_code
    };

    let reopened = PublishStore::open(config).unwrap();
    assert_eq!(
        reopened.receive(&publish_code).unwrap().remaining_receives,
        0
    );
    assert!(reopened.receive(&publish_code).is_err());
}

#[test]
fn store_does_not_resurrect_exhausted_publish_after_restart() {
    let (_guard, config) = temp_config("exhausted-replay");
    let payload = contact_payload("exhausted-replay");
    let publish_code = {
        let store = PublishStore::open(config.clone()).unwrap();
        let create = create_publish(&store, 900, 1, &payload, "exhausted-replay@example.test");
        verify_publish(&store, &create);
        assert_eq!(
            store
                .receive(&create.publish_code)
                .unwrap()
                .remaining_receives,
            0
        );
        create.publish_code
    };

    let reopened = PublishStore::open(config).unwrap();
    assert!(reopened.receive(&publish_code).is_err());
}

#[test]
fn single_use_publish_is_removed_as_soon_as_it_is_received() {
    let (_guard, config) = temp_config("single-use-delete");
    let payload = contact_payload("single-use-delete");
    let publish_code = {
        let store = PublishStore::open(config.clone()).unwrap();
        let create = create_publish(&store, 900, 0, &payload, "single-use-delete@example.test");
        verify_publish(&store, &create);
        assert_eq!(create.max_receives, 1);
        assert_eq!(
            store
                .receive(&create.publish_code)
                .unwrap()
                .remaining_receives,
            0
        );
        assert_eq!(store.stats().live, 0);
        create.publish_code
    };

    let reopened = PublishStore::open(config).unwrap();
    assert!(reopened.receive(&publish_code).is_err());
    assert_eq!(reopened.stats().live, 0);
}

#[test]
fn store_replays_large_persistent_store() {
    let (_guard, config) = temp_config("large-replay");
    let mut expected = Vec::with_capacity(20_000);
    {
        let store = PublishStore::open(config.clone()).unwrap();
        for index in 0..20_000_u32 {
            let payload = contact_payload(&format!("large-{index}"));
            let create = create_publish(
                &store,
                900,
                1,
                &payload,
                &format!("large-{index}@example.test"),
            );
            verify_publish(&store, &create);
            if index % 997 == 0 {
                expected.push((create.publish_code, payload));
            }
        }
        assert_eq!(store.stats().live, 20_000);
    }

    let reopened = PublishStore::open(config).unwrap();
    assert_eq!(reopened.stats().live, 20_000);
    for (publish_code, payload) in expected {
        assert_eq!(reopened.receive(&publish_code).unwrap().payload, payload);
    }
}

#[test]
fn compaction_removes_tombstoned_single_use_backlog() {
    let (_guard, config) = temp_config("compact-empty");
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("compact-empty");
    for index in 0..1_000 {
        let create = create_publish(
            &store,
            900,
            1,
            &payload,
            &format!("compact-empty-{index}@example.test"),
        );
        verify_publish(&store, &create);
        assert!(store.receive(&create.publish_code).is_ok());
    }

    assert_eq!(store.stats().live, 0);
    assert!(store.stats().segment_bytes > 0);
    let report = store.compact().unwrap();
    assert!(report.bytes_before > report.bytes_after);
    assert_eq!(store.stats().segment_bytes, 0);
}

#[test]
fn compaction_preserves_live_records() {
    let (_guard, config) = temp_config("compact-live");
    let store = PublishStore::open(config).unwrap();
    let mut live = Vec::new();
    for index in 0..1_000_u32 {
        let payload = contact_payload(&format!("compact-live-{index}"));
        let create = create_publish(
            &store,
            900,
            2,
            &payload,
            &format!("compact-live-{index}@example.test"),
        );
        verify_publish(&store, &create);
        if index % 100 == 0 {
            live.push((create.publish_code, payload));
        }
    }

    let before = store.stats().segment_bytes;
    let report = store.compact().unwrap();
    assert!(report.bytes_after <= before);
    for (publish_code, payload) in live {
        assert_eq!(store.receive(&publish_code).unwrap().payload, payload);
    }
}

#[test]
fn encoded_requests_are_accepted_by_store_handler() {
    let (_guard, config) = temp_config("handler");
    let store = PublishStore::open(config).unwrap();
    let payload = contact_payload("handler");

    let request = decode_request(
        &lockbox_publish_protocol::protocol::encode_publish_request_with_email(
            900,
            1,
            &payload,
            Some("handler@example.test"),
        ),
        2048,
    )
    .unwrap();
    let response = store.handle(request.operation, &request.payload);
    assert_success(&response);
    let mut reader = Reader::new(&response[14..]);
    reader.message_version().unwrap();
    let publish_code = reader.string().unwrap();
    let delete_token = reader.bytes().unwrap();
    let _expires_at_ms = reader.u64().unwrap();
    let _max_receives = reader.u16().unwrap();
    let verification_url = reader.string().unwrap();
    let (code, token) = verification_query_parts(&verification_url);
    assert_eq!(code, publish_code);
    assert!(store.verify_email(&code, &token).success);

    let request = decode_request(&encode_receive_request(&publish_code), 1024).unwrap();
    let response = store.handle(request.operation, &request.payload);
    assert_success(&response);

    let request =
        decode_request(&encode_delete_request(&publish_code, &delete_token), 1024).unwrap();
    let response = store.handle(request.operation, &request.payload);
    assert_success(&response);
}

#[test]
#[ignore = "requires local TCP sockets, which are blocked in the test sandbox"]
fn client_api_can_receive_publish_and_delete() {
    if !has_loopback_sockets() {
        eprintln!("skipping local-socket e2e test in restricted environment");
        return;
    }
    let (_guard, config) = temp_config("client-api");
    let store = Arc::new(PublishStore::open(config).unwrap());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server_store = Arc::clone(&store);
    thread::spawn(move || {
        let _ = lockbox_key_server::server::run_listener(listener, server_store);
    });
    thread::sleep(Duration::from_millis(50));

    let client = PublishClient::new(&format!("http://{addr}/v1/publish")).unwrap();
    let published = client
        .publish_contact(
            900,
            2,
            ContactPublish {
                identity: "client@example.com",
                public_key: b"public-key-material",
                signing_public_key: b"signing-public-key-material",
                fingerprint: &[1_u8; 32],
                publish_nonce: &[2_u8; 24],
                created_at_unix_ms: 1,
                expires_at_unix_ms: 2,
                verification_email: Some("client@example.com"),
            },
        )
        .unwrap();
    assert_eq!(published.max_receives, 2);
    let (code, token) = verification_query_parts(published.verification_url.as_deref().unwrap());
    assert_eq!(code, published.publish_code);
    assert!(store.verify_email(&code, &token).success);

    let received = client.receive(&published.publish_code).unwrap();
    assert_eq!(
        received.payload_type,
        lockbox_publish_protocol::payload::PayloadType::ContactPublish
    );
    assert_eq!(received.remaining_receives, 1);

    assert!(client
        .delete(&published.publish_code, &published.delete_token)
        .unwrap());
}

fn has_loopback_sockets() -> bool {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => false,
        Err(error) => panic!("unable to bind 127.0.0.1:0 for local e2e server: {error}"),
    }
}

fn assert_success(response: &[u8]) {
    assert_status(response, Status::Success);
}

fn assert_status(response: &[u8], status: Status) {
    assert_eq!(&response[0..4], b"LBSR");
    assert_eq!(
        u16::from_be_bytes([response[6], response[7]]),
        status as u16
    );
}

fn temp_config(name: &str) -> (TempGuard, ServerConfig) {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "lockbox-key-server-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    let config = ServerConfig {
        bind_addr: "127.0.0.1:0".to_string(),
        state_dir: path.clone(),
        server_id: 0,
        cluster_id: "default".to_string(),
        public_url: None,
        topology_version: 1,
        topology_servers: Vec::new(),
        topology_routes: Vec::new(),
        replication_token: None,
        topology_token: None,
        topology_stale_after_ms: 90_000,
        topology_heartbeat_interval_ms: 30_000,
        replication_peer_urls: Vec::new(),
        origin_epoch: 1,
        promoted_owner_ids: Vec::new(),
        max_payload_bytes: 1024,
        verification_ttl: Duration::from_secs(1800),
        default_receive_ttl: Duration::from_secs(7200),
        max_receive_ttl: Duration::from_secs(7200),
        shard_count: 4,
        developer_mode: true,
        benchmark_requests: 50_000,
        benchmark_payload_bytes: 512,
        benchmark_concurrency: 0,
        benchmark_preload_published_payloads: 0,
        max_receives_per_publish: 8,
        compact_min_bytes: 1,
        index_cache_entries: 65_536,
        rate_limit_per_minute: 120,
        rate_limit_burst: 40,
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from: None,
        smtp_tls: SmtpTlsMode::StartTls,
        smtp_timeout: Duration::from_secs(300),
        verification_email_subject: "Verify {publish_code}".to_string(),
        verification_email_template: "Verify {email}: {verification_url}".to_string(),
        verification_email_rate_limit_per_hour: 5,
        verification_email_ip_rate_limit_per_hour: 30,
    };
    (TempGuard(path), config)
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

struct TempGuard(PathBuf);

impl Drop for TempGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
