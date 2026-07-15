use std::time::Duration;

use revault_publish_protocol::client::{ContactPublish, PublishClient};
use revault_publish_protocol::protocol::{self, Operation, Status};

use super::support::{
    contact_publish_payload, publish_success_response, FlakyTransport, MockTransport,
};

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
                profile: "client@example.com",
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
        revault_publish_protocol::PayloadType::ContactPublish
    );
    assert_eq!(received.remaining_receives, 0);

    assert!(client
        .delete(&published.publish_code, &published.delete_token)
        .unwrap());
}

#[test]
fn client_retries_transient_transport_errors() {
    let transport = FlakyTransport::new(publish_success_response("123456789012"));
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
                profile: "client@example.com",
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
fn http_transport_accepts_https_urls() {
    assert!(revault_publish_protocol::HttpTransport::new(
        "https://keypublish.revault.onepub.dev/v1/publish"
    )
    .is_ok());
    assert!(
        revault_publish_protocol::HttpTransport::new("https://keypublish.revault.onepub.dev")
            .is_ok()
    );
    assert!(
        revault_publish_protocol::HttpTransport::new("ftp://keypublish.revault.onepub.dev")
            .is_err()
    );
}
