use lockbox_publish_protocol::client::{PublishClient, PublishClientPool};
use lockbox_publish_protocol::protocol::{self, Operation, Status};
use lockbox_publish_protocol::TopologyRoute;

use super::support::{
    contact_publish_payload, delete_success_response, one_attempt_client, publish_success_response,
    rate_limited_response, receive_success_response, MockTransport,
};

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
fn client_pool_receives_from_later_server_when_first_misses() {
    let payload = contact_publish_payload();
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");

    let client = PublishClientPool::from_transports(vec![
        MockTransport::new(vec![protocol::encode_response(
            Operation::Receive,
            Status::PublishNotFound,
            &error,
        )]),
        MockTransport::new(vec![receive_success_response(&payload)]),
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
    let response = receive_success_response(&contact_publish_payload());
    let server_0 = MockTransport::new(vec![response.clone()]);
    let server_1 = MockTransport::new(vec![response]);

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
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");
    let receive_response = receive_success_response(&contact_publish_payload());
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
    let server_2 = MockTransport::new(vec![receive_response]);

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
    let payload = contact_publish_payload();
    let mut error = Vec::new();
    protocol::put_u16(&mut error, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut error, Status::PublishNotFound as u16);
    protocol::put_string(&mut error, "publish not found");
    let receive_response = receive_success_response(&payload);
    let server_0 = MockTransport::new(vec![receive_response.clone()]);
    let server_1 = MockTransport::new(vec![protocol::encode_response(
        Operation::Receive,
        Status::PublishNotFound,
        &error,
    )]);
    let server_2 = MockTransport::new(vec![receive_response]);

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
fn client_pool_deletes_from_later_server_when_first_misses() {
    let client = PublishClientPool::from_transports(vec![
        MockTransport::new(vec![delete_success_response(false)]),
        MockTransport::new(vec![delete_success_response(true)]),
    ])
    .unwrap();

    assert!(client.delete("123456789012", b"delete-token").unwrap());
}
