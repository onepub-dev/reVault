use std::time::SystemTime;

use super::*;

#[test]
fn with_filtered_stale_servers_removes_stale_servers_and_routes() {
    let now_ms = unix_ms(SystemTime::now());
    let topology = ClusterTopology {
        cluster_id: "acme".to_string(),
        version: 1,
        servers: vec![
            TopologyServer {
                id: 0,
                url: "http://publish0/v1/publish".to_string(),
                status: ServerStatus::Active,
                last_seen_ms: Some(now_ms),
            },
            TopologyServer {
                id: 1,
                url: "http://publish1/v1/publish".to_string(),
                status: ServerStatus::Active,
                last_seen_ms: Some(now_ms - 200),
            },
            TopologyServer {
                id: 2,
                url: "http://publish2/v1/publish".to_string(),
                status: ServerStatus::Disabled,
                last_seen_ms: Some(now_ms),
            },
        ],
        routes: vec![
            TopologyRoute {
                owner_id: 0,
                primary_id: 0,
                failover_ids: vec![1],
            },
            TopologyRoute {
                owner_id: 1,
                primary_id: 1,
                failover_ids: vec![0],
            },
            TopologyRoute {
                owner_id: 2,
                primary_id: 2,
                failover_ids: vec![0],
            },
        ],
    };

    let filtered = topology.with_filtered_stale_servers(100);

    assert_eq!(filtered.servers.len(), 1);
    assert_eq!(filtered.servers[0].id, 0);
    assert_eq!(
        filtered.routes,
        vec![TopologyRoute {
            owner_id: 0,
            primary_id: 0,
            failover_ids: vec![0],
        }],
    );
}

#[test]
fn build_ring_routes_ignores_inactive_servers() {
    let routes = build_ring_routes(&[
        TopologyServer {
            id: 0,
            url: "http://publish0/v1/publish".to_string(),
            status: ServerStatus::Active,
            last_seen_ms: Some(10),
        },
        TopologyServer {
            id: 1,
            url: "http://publish1/v1/publish".to_string(),
            status: ServerStatus::Disabled,
            last_seen_ms: Some(20),
        },
        TopologyServer {
            id: 2,
            url: "http://publish2/v1/publish".to_string(),
            status: ServerStatus::Standby,
            last_seen_ms: Some(30),
        },
    ]);

    assert_eq!(
        routes,
        vec![
            TopologyRoute {
                owner_id: 0,
                primary_id: 0,
                failover_ids: vec![2],
            },
            TopologyRoute {
                owner_id: 2,
                primary_id: 2,
                failover_ids: vec![0],
            },
        ]
    );
}
