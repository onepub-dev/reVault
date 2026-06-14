use std::cell::RefCell;
use std::time::Duration;

use lockbox_publish_protocol::client::{
    ContactPublish, PublishClient, PublishClientPool, Transport,
};
use lockbox_publish_protocol::protocol::{self, Operation, Status};
use lockbox_publish_protocol::{
    decode_replication_request, encode_replication_request, sign_replication_event,
    KeyServerStatus, ReplicationEvent, ReplicationEventKind, ReplicationRequest,
};
use lockbox_publish_protocol::{ClusterTopology, ServerStatus, TopologyRoute, TopologyServer};

#[derive(Clone)]
struct MockTransport {
    responses: std::rc::Rc<RefCell<Vec<Vec<u8>>>>,
    calls: std::rc::Rc<RefCell<usize>>,
}

#[derive(Clone)]
struct FlakyTransport {
    response: Vec<u8>,
    calls: std::rc::Rc<RefCell<usize>>,
}

impl FlakyTransport {
    fn new(response: Vec<u8>) -> Self {
        Self {
            response,
            calls: std::rc::Rc::new(RefCell::new(0)),
        }
    }

    fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl Transport for FlakyTransport {
    fn post_binary(&self, _body: &[u8]) -> Result<Vec<u8>, lockbox_publish_protocol::ClientError> {
        let mut calls = self.calls.borrow_mut();
        *calls += 1;
        if *calls == 1 {
            return Err(lockbox_publish_protocol::ClientError::Io(
                std::io::ErrorKind::WouldBlock.into(),
            ));
        }
        Ok(self.response.clone())
    }
}

impl MockTransport {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        Self {
            responses: std::rc::Rc::new(RefCell::new(responses)),
            calls: std::rc::Rc::new(RefCell::new(0)),
        }
    }

    fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl Transport for MockTransport {
    fn post_binary(&self, _body: &[u8]) -> Result<Vec<u8>, lockbox_publish_protocol::ClientError> {
        *self.calls.borrow_mut() += 1;
        Ok(self.responses.borrow_mut().remove(0))
    }
}

fn contact_publish_payload() -> Vec<u8> {
    lockbox_publish_protocol::encode_contact_publish(
        "client@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    )
}

fn publish_success_response(publish_code: &str) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_string(&mut body, publish_code);
    protocol::put_bytes(&mut body, b"delete-token");
    protocol::put_u64(&mut body, 2);
    protocol::put_u16(&mut body, 1);
    protocol::encode_response(Operation::Publish, Status::Success, &body)
}

fn receive_success_response(payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut body, payload);
    protocol::put_u64(&mut body, 2);
    protocol::put_u16(&mut body, 0);
    protocol::encode_response(Operation::Receive, Status::Success, &body)
}

fn delete_success_response(deleted: bool) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    body.push(u8::from(deleted));
    protocol::encode_response(Operation::Delete, Status::Success, &body)
}

fn rate_limited_response(operation: Operation) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut body, Status::RateLimited as u16);
    protocol::put_string(&mut body, "rate limited");
    protocol::encode_response(operation, Status::RateLimited, &body)
}

fn one_attempt_client(transport: MockTransport) -> PublishClient<MockTransport> {
    PublishClient::from_transport(transport).with_retry_policy(1, Duration::ZERO, Duration::ZERO)
}

#[test]
fn client_decodes_receive_publish_and_delete_responses() {
    let payload = contact_publish_payload();
    let mut publish_response = Vec::new();
    protocol::put_u16(&mut publish_response, protocol::MESSAGE_VERSION);
    protocol::put_string(&mut publish_response, "123456789012");
    protocol::put_bytes(&mut publish_response, b"delete-token");
    protocol::put_u64(&mut publish_response, 2);
    protocol::put_u16(&mut publish_response, 1);

    let mut receive_response = Vec::new();
    protocol::put_u16(&mut receive_response, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut receive_response, &payload);
    protocol::put_u64(&mut receive_response, 2);
    protocol::put_u16(&mut receive_response, 0);

    let mut delete_response = Vec::new();
    protocol::put_u16(&mut delete_response, protocol::MESSAGE_VERSION);
    delete_response.push(1);

    let transport = MockTransport::new(vec![
        protocol::encode_response(Operation::Publish, Status::Success, &publish_response),
        protocol::encode_response(Operation::Receive, Status::Success, &receive_response),
        protocol::encode_response(Operation::Delete, Status::Success, &delete_response),
    ]);
    let client = PublishClient::from_transport(transport);

    let published = client
        .publish_contact(
            900,
            1,
            ContactPublish {
                identity: "client@example.com",
                public_key: b"public-key-material",
                signing_public_key: b"signing-public-key-material",
                fingerprint: &[1_u8; 32],
                publish_nonce: &[2_u8; 24],
                created_at_unix_ms: 1,
                expires_at_unix_ms: 2,
                verification_email: None,
            },
        )
        .unwrap();
    assert_eq!(published.publish_code, "123456789012");

    let received = client.receive(&published.publish_code).unwrap();
    assert_eq!(
        received.payload_type,
        lockbox_publish_protocol::PayloadType::ContactPublish
    );
    assert_eq!(received.remaining_receives, 0);

    assert!(client
        .delete(&published.publish_code, &published.delete_token)
        .unwrap());
}

#[test]
fn client_pool_reuses_selected_publish_server_for_sticky_window() {
    let server_0 = MockTransport::new(vec![
        publish_success_response("00123456789012"),
        publish_success_response("00123456789013"),
    ]);
    let server_1 = MockTransport::new(vec![
        publish_success_response("11123456789012"),
        publish_success_response("11123456789013"),
    ]);
    let client =
        PublishClientPool::from_transports(vec![server_0.clone(), server_1.clone()]).unwrap();
    let payload = contact_publish_payload();

    let first = client.publish_payload(900, 1, &payload).unwrap();
    let second = client.publish_payload(900, 1, &payload).unwrap();

    assert!(client.sticky_server().unwrap().is_some());
    assert_eq!(server_0.calls() + server_1.calls(), 2);
    assert!(
        (server_0.calls() == 2 && server_1.calls() == 0)
            || (server_0.calls() == 0 && server_1.calls() == 2),
        "publish calls should stay on one sticky server"
    );
    assert_eq!(&first.publish_code[..1], &second.publish_code[..1]);
}

#[test]
fn client_pool_honors_persisted_sticky_publish_server() {
    let server_0 = MockTransport::new(vec![publish_success_response("00123456789012")]);
    let server_1 = MockTransport::new(vec![publish_success_response("11123456789012")]);
    let client =
        PublishClientPool::from_transports(vec![server_0.clone(), server_1.clone()]).unwrap();
    client.set_sticky_server(1, u64::MAX).unwrap();

    let published = client
        .publish_payload(900, 1, &contact_publish_payload())
        .unwrap();

    assert_eq!(published.publish_code, "11123456789012");
    assert_eq!(server_0.calls(), 0);
    assert_eq!(server_1.calls(), 1);
}

#[test]
fn client_pool_does_not_fail_over_after_publish_rate_limit() {
    let server_0 = MockTransport::new(vec![rate_limited_response(Operation::Publish)]);
    let server_1 = MockTransport::new(vec![publish_success_response("11123456789012")]);
    let client = PublishClientPool::from_clients(vec![
        one_attempt_client(server_0.clone()),
        one_attempt_client(server_1.clone()),
    ])
    .unwrap();
    client.set_sticky_server(0, u64::MAX).unwrap();

    let err = client
        .publish_payload(900, 1, &contact_publish_payload())
        .unwrap_err();

    assert!(err.to_string().contains("RateLimited"));
    assert_eq!(server_0.calls(), 1);
    assert_eq!(server_1.calls(), 0);
}

#[test]
fn client_pool_does_not_fail_over_after_receive_rate_limit() {
    let server_0 = MockTransport::new(vec![rate_limited_response(Operation::Receive)]);
    let server_1 = MockTransport::new(vec![receive_success_response(&contact_publish_payload())]);
    let client = PublishClientPool::from_clients(vec![
        one_attempt_client(server_0.clone()),
        one_attempt_client(server_1.clone()),
    ])
    .unwrap();

    let err = client.receive("00123456789012").unwrap_err();

    assert!(err.to_string().contains("RateLimited"));
    assert_eq!(server_0.calls(), 1);
    assert_eq!(server_1.calls(), 0);
}

#[test]
fn client_pool_does_not_fail_over_after_delete_rate_limit() {
    let server_0 = MockTransport::new(vec![rate_limited_response(Operation::Delete)]);
    let server_1 = MockTransport::new(vec![delete_success_response(true)]);
    let client = PublishClientPool::from_clients(vec![
        one_attempt_client(server_0.clone()),
        one_attempt_client(server_1.clone()),
    ])
    .unwrap();

    let err = client
        .delete("00123456789012", b"delete-token")
        .unwrap_err();

    assert!(err.to_string().contains("RateLimited"));
    assert_eq!(server_0.calls(), 1);
    assert_eq!(server_1.calls(), 0);
}

#[test]
fn client_retries_transient_transport_errors() {
    let mut publish_response = Vec::new();
    protocol::put_u16(&mut publish_response, protocol::MESSAGE_VERSION);
    protocol::put_string(&mut publish_response, "123456789012");
    protocol::put_bytes(&mut publish_response, b"delete-token");
    protocol::put_u64(&mut publish_response, 2);
    protocol::put_u16(&mut publish_response, 1);

    let transport = FlakyTransport::new(protocol::encode_response(
        Operation::Publish,
        Status::Success,
        &publish_response,
    ));
    let client = PublishClient::from_transport(transport.clone()).with_retry_policy(
        2,
        Duration::ZERO,
        Duration::ZERO,
    );

    let published = client
        .publish_contact(
            900,
            1,
            ContactPublish {
                identity: "client@example.com",
                public_key: b"public-key-material",
                signing_public_key: b"signing-public-key-material",
                fingerprint: &[1_u8; 32],
                publish_nonce: &[2_u8; 24],
                created_at_unix_ms: 1,
                expires_at_unix_ms: 2,
                verification_email: None,
            },
        )
        .unwrap();

    assert_eq!(published.publish_code, "123456789012");
    assert_eq!(transport.calls(), 2);
}

#[test]
fn client_surfaces_versioned_server_errors() {
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");

    let client =
        PublishClient::from_transport(MockTransport::new(vec![protocol::encode_response(
            Operation::Receive,
            Status::PublishNotFound,
            &error,
        )]));
    let err = client.receive("123456789012").unwrap_err();

    assert!(err.to_string().contains("PublishNotFound"));
    assert!(err.to_string().contains("publish not found"));
}

#[test]
fn client_pool_receives_from_later_server_when_first_misses() {
    let payload = lockbox_publish_protocol::encode_contact_publish(
        "cluster@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    );
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");
    let mut receive_response = Vec::new();
    protocol::put_u16(&mut receive_response, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut receive_response, &payload);
    protocol::put_u64(&mut receive_response, 2);
    protocol::put_u16(&mut receive_response, 0);

    let client = PublishClientPool::from_transports(vec![
        MockTransport::new(vec![protocol::encode_response(
            Operation::Receive,
            Status::PublishNotFound,
            &error,
        )]),
        MockTransport::new(vec![protocol::encode_response(
            Operation::Receive,
            Status::Success,
            &receive_response,
        )]),
    ])
    .unwrap();

    let received = client.receive("123456789012").unwrap();
    assert_eq!(
        received.payload_type,
        lockbox_publish_protocol::PayloadType::ContactPublish
    );
}

#[test]
fn client_pool_prefers_server_id_from_publish_code_prefix() {
    let payload = lockbox_publish_protocol::encode_contact_publish(
        "cluster@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    );
    let mut receive_response = Vec::new();
    protocol::put_u16(&mut receive_response, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut receive_response, &payload);
    protocol::put_u64(&mut receive_response, 2);
    protocol::put_u16(&mut receive_response, 0);
    let server_0 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::Success,
        &receive_response,
    )]);
    let server_1 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::Success,
        &receive_response,
    )]);

    let client =
        PublishClientPool::from_transports(vec![server_0.clone(), server_1.clone()]).unwrap();

    let received = client.receive("11123456789012").unwrap();
    assert_eq!(
        received.payload_type,
        lockbox_publish_protocol::PayloadType::ContactPublish
    );
    assert_eq!(server_0.calls(), 0);
    assert_eq!(server_1.calls(), 1);
}

#[test]
fn client_pool_prefers_locator_primary_then_secondary() {
    let payload = lockbox_publish_protocol::encode_contact_publish(
        "cluster@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    );
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");
    let mut receive_response = Vec::new();
    protocol::put_u16(&mut receive_response, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut receive_response, &payload);
    protocol::put_u64(&mut receive_response, 2);
    protocol::put_u16(&mut receive_response, 0);
    let server_0 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::PublishNotFound,
        &error,
    )]);
    let server_1 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::PublishNotFound,
        &error,
    )]);
    let server_2 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::Success,
        &receive_response,
    )]);

    let client = PublishClientPool::from_clients_with_ids(
        vec![
            PublishClient::from_transport(server_0.clone()),
            PublishClient::from_transport(server_1.clone()),
            PublishClient::from_transport(server_2.clone()),
        ],
        vec![0, 1, 2],
        Vec::new(),
    )
    .unwrap();

    let received = client.receive("12000000000000").unwrap();
    assert_eq!(received.remaining_receives, 0);
    assert_eq!(server_0.calls(), 0);
    assert_eq!(server_1.calls(), 1);
    assert_eq!(server_2.calls(), 1);
}

#[test]
fn client_pool_uses_topology_failover_order() {
    let payload = lockbox_publish_protocol::encode_contact_publish(
        "cluster@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    );
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");
    let mut receive_response = Vec::new();
    protocol::put_u16(&mut receive_response, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut receive_response, &payload);
    protocol::put_u64(&mut receive_response, 2);
    protocol::put_u16(&mut receive_response, 0);
    let server_0 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::Success,
        &receive_response,
    )]);
    let server_1 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::PublishNotFound,
        &error,
    )]);
    let server_2 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::Success,
        &receive_response,
    )]);

    let client = PublishClientPool::from_clients_with_ids(
        vec![
            PublishClient::from_transport(server_0.clone()),
            PublishClient::from_transport(server_1.clone()),
            PublishClient::from_transport(server_2.clone()),
        ],
        vec![0, 1, 2],
        vec![TopologyRoute {
            owner_id: 1,
            primary_id: 1,
            failover_ids: vec![2],
        }],
    )
    .unwrap();

    assert_eq!(client.receive("11123456789012").unwrap().payload, payload);
    assert_eq!(server_0.calls(), 0);
    assert_eq!(server_1.calls(), 1);
    assert_eq!(server_2.calls(), 1);
}

#[test]
fn topology_binary_round_trips_and_validates_routes() {
    let topology = ClusterTopology {
        cluster_id: "acme".to_string(),
        version: 42,
        servers: vec![
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
        ],
        routes: vec![TopologyRoute {
            owner_id: 0,
            primary_id: 0,
            failover_ids: vec![1],
        }],
    };

    let bytes = lockbox_publish_protocol::encode_topology(&topology).unwrap();
    let decoded = lockbox_publish_protocol::decode_topology(&bytes).unwrap();
    assert_eq!(decoded, topology);
    assert_eq!(
        decoded.urls_for_publish_code("00123456789012"),
        vec![
            "http://publish0.example/v1/publish".to_string(),
            "http://publish1.example/v1/publish".to_string()
        ]
    );
}

#[test]
fn topology_cache_round_trips_binary_documents() {
    let topology = ClusterTopology::single_server(0, "http://publish0.example/v1/publish");
    let path = std::env::temp_dir().join(format!(
        "lockbox-publish-topology-cache-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    lockbox_publish_protocol::write_topology_cache(&path, &topology).unwrap();

    let cached = lockbox_publish_protocol::read_topology_cache(&path, Duration::from_secs(60))
        .unwrap()
        .unwrap();
    assert_eq!(cached, topology);
    assert!(
        lockbox_publish_protocol::read_topology_cache(&path, Duration::from_millis(1))
            .unwrap()
            .is_some()
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn replication_request_round_trips_binary_events() {
    let event = ReplicationEvent {
        origin_server_id: 2,
        origin_epoch: 3,
        origin_sequence: 4,
        kind: ReplicationEventKind::PutPublish {
            publish_code: "21123456789012".to_string(),
            delete_token_hash: vec![9_u8; 16],
            payload: lockbox_publish_protocol::encode_contact_publish(
                "replica@example.com",
                b"public-key-material",
                b"signing-public-key-material",
                &[1_u8; 32],
                &[2_u8; 24],
                1,
                2,
            ),
            contact_email: Some("replica@example.com".to_string()),
            expires_at_unix_ms: 123,
            receive_ttl_ms: 456,
            email_verified_at_unix_ms: 789,
            max_receives: 2,
            receives: 1,
        },
    };
    let request = ReplicationRequest {
        authentication: sign_replication_event(b"peer-secret", &event),
        event,
    };

    let envelope = protocol::decode_request(&encode_replication_request(&request), 4096).unwrap();
    assert_eq!(envelope.operation, Operation::Replicate);
    assert_eq!(
        decode_replication_request(&envelope.payload).unwrap(),
        request
    );
}

#[test]
fn server_status_round_trips_binary_documents() {
    let status = KeyServerStatus {
        created: 1,
        received: 2,
        deleted: 3,
        expired: 4,
        misses: 5,
        live: 6,
        segment_bytes: 7,
        replication_pending: 8,
        replication_last_sequence: 9,
    };
    let bytes = lockbox_publish_protocol::encode_status(&status);
    assert_eq!(
        lockbox_publish_protocol::decode_status(&bytes).unwrap(),
        status
    );
}

#[test]
fn http_transport_accepts_https_urls() {
    assert!(lockbox_publish_protocol::HttpTransport::new(
        "https://keypublish.revault.onepub.dev/v1/publish"
    )
    .is_ok());
    assert!(
        lockbox_publish_protocol::HttpTransport::new("https://keypublish.revault.onepub.dev")
            .is_ok()
    );
    assert!(
        lockbox_publish_protocol::HttpTransport::new("ftp://keypublish.revault.onepub.dev")
            .is_err()
    );
}

#[test]
fn client_pool_deletes_from_later_server_when_first_misses() {
    let mut delete_miss = Vec::new();
    protocol::put_u16(&mut delete_miss, protocol::MESSAGE_VERSION);
    delete_miss.push(0);
    let mut delete_success = Vec::new();
    protocol::put_u16(&mut delete_success, protocol::MESSAGE_VERSION);
    delete_success.push(1);

    let client = PublishClientPool::from_transports(vec![
        MockTransport::new(vec![protocol::encode_response(
            Operation::Delete,
            Status::Success,
            &delete_miss,
        )]),
        MockTransport::new(vec![protocol::encode_response(
            Operation::Delete,
            Status::Success,
            &delete_success,
        )]),
    ])
    .unwrap();

    assert!(client.delete("123456789012", b"delete-token").unwrap());
}
