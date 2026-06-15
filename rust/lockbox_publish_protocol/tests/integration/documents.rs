use std::time::Duration;

use lockbox_publish_protocol::protocol::{self, Operation};
use lockbox_publish_protocol::{
    decode_replication_request, encode_replication_request, sign_replication_event,
    ClusterTopology, KeyServerStatus, ReplicationEvent, ReplicationEventKind, ReplicationRequest,
    ServerStatus, TopologyRoute, TopologyServer,
};

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
