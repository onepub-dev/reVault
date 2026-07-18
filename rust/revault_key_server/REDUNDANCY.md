# Redundant Key Server Design

This document describes the current ownership, discovery, replication, and
promotion model. Read the [server introduction](README.md) first, use the
[configuration guide](KEY_SERVER_CONFIG.md) to deploy it, and use
[Troubleshooting](TROUBLESHOOTING.md) during operations.

## Goals

The published payload service supports a simple single-server deployment while
keeping the wire format and publish codes compatible with redundant deployment.

The redundancy design must:

- keep standalone mode available for private or initial deployments
- avoid DNS round-robin sending receives to the wrong server
- support more than two servers later
- provide a clear standby and recovery path
- avoid replication storms
- preserve single-use publish semantics

## Single-server baseline

The original production deployment used one server:

```text
server_id: 0
url: https://keyshare0.revault.onepub.dev/v1/publish
```

The server still generates self-routing publish codes:

```text
0 123456789012
^ ^^^^^^^^^^^^
| random code body
server routing digit
```

The first digit is the server id. The remaining digits are the random
rendezvous code body. With the default body length of 12 digits, the displayed
code is 13 decimal digits.

Using the routing digit from day one means existing pending published payloads remain
compatible when extra servers are added later.

## Server Ids

Server ids are values from `0` to `35`, displayed as `0..9` and `a..z` in
publish codes.

The original deployment used decimal ids only. The current code format retains
those ids and extends the available owner slots with lower-case letters.

The server id is stable operational configuration, not a random instance id.
A replacement machine for server `3` must run with `server_id = 3` when it is
serving published payloads owned by that id.

## Client Routing

Users should not have to configure failover pairs. Failover topology belongs on
the servers.

Every key-server member serves the topology document. There is no separate
topology service. For an organization running its own publish service, clients
can be configured with a topology URL:

```yaml
publish:
  topology_url: "https://keyshare0.example.com/v1/topology"
```

The production reVault CLI has two ordered bootstrap URLs,
`keyshare0.revault.onepub.dev/v1/topology` and
`keyshare1.revault.onepub.dev/v1/topology`. It tries the second if the first is
unavailable, then learns all current member URLs from the returned document.

The topology endpoint returns public routing metadata as a binary document:

```text
TopologyDocument {
  magic: "LBST"
  version: u16
  cluster_id: utf8_string
  topology_version: u64
  server_count: u16
  servers: TopologyServer[server_count]
  route_count: u16
  routes: TopologyRoute[route_count]
}

TopologyServer {
  id: u8
  status: u8
  url: utf8_string
}

TopologyRoute {
  owner_id: u8
  primary_id: u8
  failover_count: u16
  failover_ids: u8[failover_count]
}
```

The operator configures this topology on the key servers:

```yaml
key_servers:
  - id: 0
    url: https://keyshare0.revault.onepub.dev/v1/publish
  - id: 1
    url: https://keyshare1.revault.onepub.dev/v1/publish
  - id: 2
    url: https://keyshare2.revault.onepub.dev/v1/publish
```

For `PUBLISH`, the CLI selects a key-server member from the discovered topology
and keeps that choice sticky for 24 hours. The sticky selection is persisted
locally so repeated CLI invocations normally use the same member instead of
spreading requests across the cluster. The selected member generates a code
prefixed with its own id.

For `RECEIVE` and `DELETE`, the CLI reads the first digit and sends the request
to the primary endpoint for that owner id first. It then tries the failover
list from topology. If the topology is missing or stale, the client may try
every endpoint in the same discovered cluster.

If any publish, receive, or delete attempt receives a `RateLimited` response,
the CLI does not fail over to another server for that operation. This prevents
clients from bypassing per-server limits by hopping across the cluster.

Servers also turn an anonymous-client rate-limit violation into a 24 hour
cluster block for that source IP. The block is held locally before the token
bucket is checked and is replicated to peers over the authenticated replication
channel, so a client cannot wait for a different server's bucket and retry
there. Server-token authenticated topology and replication traffic is exempt
from this client block.

Trying every server is acceptable as a fallback within one trusted cluster. It
must not spray publish codes across unrelated public services.

DNS may still be used for normal host resolution and coarse failover, but DNS
round-robin is not the primary routing mechanism. Without self-routing codes or
replicated state, DNS round-robin can randomly send receives to a server that
does not own the published payload.

## Standby Replication

The first redundancy step after standalone mode is paired or ring standby
replication.

Two servers:

```text
0 -> 1
1 -> 0
```

Three servers:

```text
0 -> 1
1 -> 2
2 -> 0
```

Each server is authoritative for its own prefixed codes. It streams state
events to its standby peer. The standby stores those events as replica state for
the original owner id.

The standby does not normally serve replicated published payloads. It serves them only when
it is explicitly promoted for that owner id.

## Replication Events

Replication must be based on append-log events, not on replaying client HTTP
requests.

The required event types are:

```text
put_publish
receive_count
consume_publish
delete_publish
expire_publish
rate_limit_block
```

The replication envelope must include:

```text
origin_server_id
origin_epoch
origin_sequence
event_type
event_body
message_authentication
```

The idempotency key is:

```text
(origin_server_id, origin_epoch, origin_sequence)
```

Each standby tracks the last applied sequence per origin. Replayed or duplicate
events are ignored.

## No Replication Storms

Client and replication traffic must be separate operations:

```text
POST /v1/publish
POST /v1/replicate
```

Rules:

- client-originated publish events are appended locally and queued for replication
- peer-originated replication events are applied idempotently
- replicated events are not re-replicated by default
- chain replication is out of scope for the first redundant version

This prevents a two-server pair from bouncing the same event forever and
prevents a ring from amplifying each published payload into a storm.

The current implementation sends `put_publish`, `receive_count`, `delete`
tombstone, and `rate_limit_block` events to configured peers. The standby
applies peer events through `/v1/replicate` and does not enqueue those peer
events for replication again. Expired published payloads are still rejected by
timestamp when a standby is promoted, but active expiry purge events are not yet
replicated as first-class events.

## Failover

Only one server may be authoritative for a server id at a time.

If server `2` fails:

1. Its standby is promoted for owner id `2`.
2. The topology endpoint maps owner id `2` to the promoted standby.
3. The old server `2`, if it returns, starts non-authoritative.
4. The old server resyncs from the promoted node before serving owner id `2`.

Automatic dual-serving is not allowed in the first redundant design because it
can duplicate single-use receives during partial network failures.

## Recovery

A promoted standby must persist:

- the replicated publish records
- consumed/deleted tombstones
- the origin epoch and sequence position

When an old primary returns, recovery is a controlled operator action:

```text
stop old primary serving owner id N
copy or stream missing events from promoted standby
verify sequence continuity
switch authority back only after sync is complete
```

If sequence continuity cannot be proven, the safe recovery path is to keep the
promoted standby authoritative until all pending published payloads for that owner id have
expired.

## Why Not Hot/Hot First

Hot/hot serving is harder because `RECEIVE` mutates state. A single-use publish must
not be returned by two servers during a race.

A future hot/hot design should use deterministic ownership:

```text
owner = first publish-code digit
```

Any server may accept the HTTP connection, but mutating operations for owner
`N` must be executed by the authoritative node for `N` or by a promoted standby
for `N`.

Until that routing/proxy layer exists, standby replication plus explicit
promotion is the safer design.

## Implementation Status

Implemented:

- server id configuration, defaulting to `0`
- generated publish codes include the server id prefix
- default random body length remains 12 digits
- client pool support can prefer the server encoded in the first digit
- public topology model and binary codec
- `GET /v1/topology`
- server CLI topology flags
- client pool construction from discovered topology
- client-side binary topology cache helpers
- publish client pool selection across configured/discovered servers
- `/v1/replicate`
- signed peer replication using a configured published secret
- local origin epoch and monotonically increasing origin sequence numbers
- standby idempotency tracking persisted across restarts
- promotion gating through configured promoted owner ids
- replication storm avoidance by separating client and peer operations
- self-installing systemd service support with boot enablement
- CLI YAML topology URL/default public service config
- vault CLI publish/receive/delete wiring through `PublishClientPool`
- durable outbound replication outbox with retry
- expiry/exhaustion tombstones queued for replication
- binary `/v1/status` replication lag/status reporting
- `resync-peer` operator tooling for live-published-payload replay to a peer
- signed replication envelopes
- TLS-capable client transport for the public default service

Remaining work:

- durable per-peer acknowledgements when more than one replication peer is
  configured
- full old-primary handback workflow around `resync-peer`
- mTLS support for peer authentication in addition to signed envelopes
- end-to-end operator tests for failover promotion and recovery flows
