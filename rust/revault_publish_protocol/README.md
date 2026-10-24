# revault_publish_protocol

Wire protocol types, binary encoders/decoders, and client helpers for the
reVault publish/key rendezvous service.

This crate is intended for code that needs to talk to a compatible reVault
publish server or share protocol documents with one. It contains the protocol
surface only; it does not run a server or manage vault storage.

## What It Provides

- Binary request and response encoding for publish, receive, and delete flows.
- Payload helpers for contact publishes and key replacement documents.
- A small blocking client API with HTTP transport support.
- Cluster topology encoding, decoding, routing, and cache helpers.
- Replication request encoding, decoding, and signing helpers.
- Server status document encoding and decoding.
- Reciprocal invitation records, hybrid signatures and one full shared
  verification fingerprint covering both participants' complete key bundles.
- An HTTPS invitation client with bounded messages and redirects disabled.

## Reciprocal exchange

Use `exchange::{Offer, Acceptance, LocalExchange}` for the invitation-scoped
flow. Each participant selects one profile. The recipient signs the exact
offer and returns both public keys. Each client pins its contribution and
independently compares the entire value returned by `exchange::verification`
before marking the other participant trusted. There is no receive-only mode.

The default `http` feature provides the network clients and publication
topology/replication helpers. Set `default-features = false` when using the
protocol types from the vault or WebAssembly without an HTTP/TLS dependency.

See [the protocol design](../../docs/reciprocal_exchange.md) and
[the user flow](../../manual/cli-tooling/sharing.md). The reciprocal endpoint
is `/v2/exchange`; it uses one explicitly selected relay and does not inherit
publication-cluster failover.

## Example

```rust
use revault_publish_protocol::{PublishClient, encode_contact_publish};

fn publish_contact() -> Result<(), Box<dyn std::error::Error>> {
    let client = PublishClient::new("https://keypublish.example/v1/publish")?;
    let payload = encode_contact_publish(
        "alice@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1_725_000_000_000,
        1_725_086_400_000,
    );

    let published = client.publish_payload(900, 1, &payload)?;
    println!("publish code: {}", published.publish_code);
    Ok(())
}
```

For custom transports or in-memory tests, implement the `Transport` trait and
construct a `PublishClient` with `PublishClient::from_transport`.

## Topology

Use `ClusterTopology` and the topology helpers when a client needs deterministic
routing across multiple publish servers:

```rust
use revault_publish_protocol::{ClusterTopology, decode_topology};

fn server_urls(bytes: &[u8], publish_code: &str) -> Vec<String> {
    let topology: ClusterTopology = decode_topology(bytes).unwrap();
    topology.urls_for_publish_code(publish_code)
}
```

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

This crate is distributed under the reVault Source Available License 1.0. See
`LICENSE` for the full terms.
