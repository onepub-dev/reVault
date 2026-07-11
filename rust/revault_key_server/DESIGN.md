# reVault Key Server Design

## Purpose

The key server is a high-throughput rendezvous service for short-lived
reVault contact sharing requests. It helps one client publish a candidate
public key payload and another client receive that payload by publish code.

The server must not be trusted for identity, key ownership, or verification.
It only relays candidate key material. Trust decisions remain local and
explicit in the reVault client.

Redundant deployment and failover are covered in
[`REDUNDANCY.md`](REDUNDANCY.md). The initial production deployment is still a
single server, but publish codes include a server routing digit from the start so
future multi-server deployments do not need a publish-code format break.

```text
publish/import/receive -> candidate key
verify -> trusted contact key
key change -> pending changed key requiring verification
```

## Workspace Placement

The server lives in its own workspace crate:

```text
revault_key_server/
```

This keeps HTTP, async runtime, rate limiting, deployment, and purge logic out
of `revault_lockbox_api` and `revault_vault_api`. Those crates should stay focused on
archive, key, and local vault behavior.

If the CLI and server need published wire types, move those types into:

```text
revault_publish_protocol/
```

That protocol crate should avoid HTTP dependencies so the CLI can use it
without pulling in server runtime code.

## Published Protocol Crate

`revault_publish_protocol` owns everything a client and server must agree on:

```text
request/response binary envelopes
operation body versions
typed publish payload envelopes
payload validators and encoders
response decoders
blocking client API
```

The server crate depends on `revault_publish_protocol`; it must not carry a
private duplicate of the wire format. The CLI should also depend on
`revault_publish_protocol` when it grows `lockbox vault identity publish`, `lockbox contact add
--publish-code`, and `lockbox contact update --publish-code`.

The client API should make the normal call flow explicit:

```rust
let client = PublishClient::new("http://127.0.0.1:8089/v1/publish")?;

let published = client.publish_contact(
    900,
    1,
    ContactPublish {
        identity: "alice@example.com",
        public_key: alice_public_key,
        fingerprint: alice_fingerprint,
        publish_nonce,
        created_at_unix_ms,
        expires_at_unix_ms,
    },
)?;

let received = client.receive(&published.publish_code)?;
client.delete(&published.publish_code, &published.delete_token)?;
```

`publish_payload` accepts any already encoded and validated `PublishPayload`, which
lets CLI code send signed and unsigned replacement payloads without the HTTP
client knowing contact-trust semantics. `receive` returns both the raw payload and
the validated `PayloadType` so higher layers can dispatch to contact-add,
signed-replacement, or unsigned-replacement logic.

The first client implementation uses blocking `std::net::TcpStream` for
`http://` endpoints because the key server currently implements plain HTTP
itself. Production TLS can be provided by an edge proxy, or by adding a TLS
transport implementation without changing the binary protocol or payload
validators.

## Wire Protocol

The service exposes a single binary HTTP endpoint:

```text
POST /v1/publish
```

Every request body starts with a small binary envelope. The envelope identifies
the operation and the payload encoding version. The HTTP layer only provides
transport, TLS termination, body limits, and response status. Application
successes and failures are encoded in the binary response body.

All multi-byte integers are big-endian. Strings and opaque bytes are
length-prefixed. Unknown protocol versions are rejected.

```text
RequestEnvelope {
    magic:      "LBSR"
    version:    u16
    operation:  u16
    flags:      u16
    payload_len: u32
    payload:    [u8; payload_len]
}
```

Operations:

```text
1 SHARE
2 RECEIVE
3 DELETE
```

Responses use the same shape for success and error:

```text
ResponseEnvelope {
    magic:       "LBSR"
    version:     u16
    status:      u16
    operation:   u16
    payload_len: u32
    payload:     [u8; payload_len]
}
```

Status codes:

```text
0 success
1 malformed_request
2 unsupported_version
3 unknown_operation
4 payload_too_large
5 publish_not_found
6 publish_expired
7 publish_exhausted
8 delete_token_invalid
9 rate_limited
10 store_unavailable
11 internal_error
```

Error payload:

```text
ErrorPayload {
    message_version: u16
    code:    u16
    message: utf8_string
}
```

Error messages are diagnostic only. Clients must branch on `code`, not on the
message text.

## Operations

`SHARE` stores a typed, versioned candidate payload and returns a rendezvous
code.

```text
PublishRequest {
    message_version: u16
    ttl_seconds: u32
    max_receives: u16
    payload: PublishPayload
}

PublishResponse {
    message_version: u16
    publish_code: utf8_string
    delete_token: opaque_bytes
    expires_at_unix_ms: u64
    max_receives: u16
}
```

`RECEIVE` returns the stored candidate payload if the code exists, has not
expired, and has remaining receive allowance.

```text
ReceiveRequest {
    message_version: u16
    publish_code: utf8_string
}

ReceiveResponse {
    message_version: u16
    payload: PublishPayload
    expires_at_unix_ms: u64
    remaining_receives: u16
}
```

`DELETE` revokes a published payload before expiry. The delete token is returned only to
the publishing client by `SHARE`.

```text
DeleteRequest {
    message_version: u16
    publish_code: utf8_string
    delete_token: opaque_bytes
}

DeleteResponse {
    message_version: u16
    deleted: bool
}
```

`publish_code` is a rendezvous code, not a verifier. It helps the receiving
client find one candidate payload. It does not prove who created that payload.

The default publish model is single-use. A normal contact publish should be removed
as soon as the receiver obtains it. This avoids building a large
backlog of records that will never be accessed again.

Multi-contact sharing is allowed only when the publishing client explicitly
requests a larger `max_receives`, and the server must cap that value. This lets
one user publish the same candidate key with a small group without creating a new
publish code for every contact, while keeping accidental long-lived fan-out
under control.

The production default publish code body should be 12 random decimal digits.
The displayed code includes one leading server routing digit plus that random
body, so the default displayed code is 13 decimal digits. Six random digits are
convenient for small or developer deployments, but they cap the live code space
at one million and create collision pressure under high request rates. The
server should support configurable decimal body lengths clamped to a safe range
such as 6 to 12 digits.

## Payload Model

The server stores bounded typed payloads. It validates the payload envelope,
protocol version, message type, message version, required fields, field sizes,
basic field shape, payload size, TTL, receive count, and delete-token shape. It
does not validate identity claims, public key ownership, replacement
continuity, or contact trust state.

Validating structure does not make a trust assertion. It only prevents the
key server from being a generic blob relay. The server can reject payloads
that are not exactly one of the supported Lockbox publish message formats while
still treating accepted payloads as untrusted candidate material.

Each stored payload starts with its own envelope. The outer operation body
version and the stored payload version are separate so `DELETE` and `RECEIVE`
message shapes can evolve independently from contact payload formats.

```text
PublishPayload {
    magic:       "LBSP"
    version:     u16
    message_type: u16
    body_len:    u32
    body:        [u8; body_len]
}
```

Supported message types:

```text
1 contact_publish_v1
2 signed_key_replacement_v1
3 unsigned_key_replacement_v1
```

Unsupported payload protocol versions, unknown message types, over-large
fields, missing fields, bad UTF-8, trailing bytes, and malformed timestamps are
rejected before the published payload is stored.

Not interpreting trust means:

```text
the server does not decide whether an identity is real
the server does not decide whether a key belongs to an identity
the server does not mark keys verified
the server does not compare replacement keys with local contact history
the server does not suppress key changes
the server does not issue identity assertions
```

The client owns all trust behavior. A server response is always only a
candidate key payload.

`contact_publish_v1` contains:

```text
identity
public_key
signing_public_key
public_key_fingerprint
publish_nonce
created_at_unix_ms
expires_at_unix_ms
```

`signed_key_replacement_v1` contains:

```text
identity
old_public_key_fingerprint
new_public_key
new_signing_public_key
new_public_key_fingerprint
replacement_nonce
signature_by_old_key
created_at_unix_ms
expires_at_unix_ms
```

`unsigned_key_replacement_v1` contains:

```text
identity
old_public_key_fingerprint
new_public_key
new_signing_public_key
new_public_key_fingerprint
replacement_nonce
created_at_unix_ms
expires_at_unix_ms
```

The verification code shown to users is generated by clients from the
candidate payload:

```text
hash("lockbox contact verify v1" || identity || public_key || publish_nonce)
```

The published payload code and verification code are deliberately different. The published payload code
must match on both sides because it selects the stored payload. The
verification code must be derived from the actual received payload because it is
used to detect server-side substitution.

Example:

```text
Alice uploads key A and sees verification code 71-44-92.
Bob receives the published payload code.
If the server returns key A, Bob also sees 71-44-92.
If the server substitutes key M, Bob sees a different verification code.
Alice and Bob compare verification codes over an independent channel.
```

## Durable Store

The primary store should be in-process and disk-backed. The server should not
depend on Redis, Postgres, SQLite, or another external service for the first
production design.

The store is optimized around short-lived records and single-key lookups:

```text
append-only segment files for records
disk bucket index from code_hash to record location
bounded in-memory recent-published-payload cache
in-memory expiry buckets for purge
periodic compaction for live records
```

Common operations should be O(1) expected time:

```text
SHARE   -> append record, append bucket index entry, cache recent entry
RECEIVE -> cache lookup or bucket lookup, read payload from disk offset
DELETE  -> lookup, append tombstone, remove cached entry
PURGE   -> process due expiry buckets, append tombstones, remove cached entries
```

When `RECEIVE` consumes the final allowed receive, it must append a tombstone and
remove the published payload from the live index before returning success. If receives remain,
the updated receive count must be persisted so reboot cannot restore consumed
receive allowance.

The authoritative index is on disk. The in-memory index is only a bounded
recent-published-payload cache. Several million pending published payloads are reasonable because old
pending published payloads do not require one in-memory entry each.

Each disk bucket record is fixed-size and compact:

```text
code_hash: 16 bytes
delete_token_hash: 16 bytes
payload_offset: u64
payload_len: u32
expires_at_unix_ms: u64
max_receives/receives/state
```

Lookup hashes the published payload code, selects one bucket file, and scans that compact
bucket backward until it finds the latest state for the hash. With thousands of
buckets, this avoids scanning the full store while keeping memory bounded.

## File Format

Use append-only segment files so writes are sequential and durable:

```text
published-payloads-000001.seg
published-payloads-000002.seg
...
```

Each segment contains records:

```text
RecordHeader {
    magic:       "LBSF"
    version:     u16
    kind:        u16
    header_len:  u16
    flags:       u16
    record_len:  u32
    crc32:       u32
}
```

Record kinds:

```text
1 put_publish
2 tombstone
3 receive_count
```

`put_publish` body:

```text
PutPublishRecord {
    code_hash: [u8; 32]
    delete_token_hash: [u8; 32]
    created_at_unix_ms: u64
    expires_at_unix_ms: u64
    max_receives: u16
    payload: PublishPayload
}
```

`tombstone` body:

```text
TombstoneRecord {
    code_hash: [u8; 32]
    deleted_at_unix_ms: u64
    reason: u16
}
```

`receive_count` body:

```text
ReceiveCountRecord {
    code_hash: [u8; 32]
    receives: u16
}
```

Receive count persistence must be exact for live multi-receive published payloads. Append a
`receive_count` record before returning each successful receive that leaves the
published payload live. Append a tombstone before returning a successful receive that
consumes the final allowed receive.

On startup, rebuild the in-memory index by replaying segment files in order.
Ignore expired published payloads during replay. Verify record CRCs and stop at the last
valid record if the final segment was partially written during a crash.

## Purging

Do not purge by scanning the whole index. The purge path must be proportional
to the number of expiring records, not the total number of live records.

Use fixed-width expiry buckets, for example one bucket per second or one bucket
per five seconds:

```text
bucket_index = expires_at / bucket_width
bucket[bucket_index].push(code_hash)
```

A background task advances through due buckets and removes those code hashes
from the in-memory index. It appends tombstones for removed published payloads so replay
does not resurrect deleted records.

Receives also check `expires_at` and remove stale entries opportunistically.

Compaction rewrites live, unexpired records from older segments into new
segments and then removes obsolete segment files. Compaction should run when
dead bytes exceed a configured ratio or when segment count exceeds a configured
limit.

For the common single-use model, successful receive of the only allowed contact
appends a tombstone and removes the record from the live index. Background
compaction then truncates empty shards or rewrites only live records, preventing
an unbounded on-disk backlog of already-consumed published payloads.

## Concurrency

The server should allow multiple HTTP worker tasks. Thousands of requests per
second will be easier to sustain with concurrent request parsing, rate limiting,
and disk-cache reads.

To keep the persistent store manageable, shard the store by `code_hash`:

```text
shard = first_n_bits(code_hash) % shard_count
```

Each shard owns:

```text
hash index
expiry buckets
LRU payload cache
segment writer lock
```

This avoids one global lock while keeping each shard simple. A request touches
only one shard after the code hash is known.

The append path should use a per-shard writer lock or writer task. Reads should
use the in-memory index and cache without taking the writer lock except when a
receive count must be persisted exactly.

Start with a fixed shard count configured at process startup. Do not implement
dynamic shard resizing in the first version.

## Security Controls

The published payload code is a rendezvous code, not a trust mechanism. A six-digit code
has limited entropy, so rate limiting is required. Production deployments
should use the 12-digit default unless they have a specific reason to reduce
manual-entry length.

Required controls:

```text
email verification TTL, defaulting to 30 minutes
receive TTL after email verification, defaulting to 2 hours
server-side max receive TTL, defaulting to 2 hours
payload size cap, defaulting to 8 KiB
per-IP request rate limits
per-code failed-attempt limits
small max_receives cap
delete token for early revocation
hash publish codes at rest
constant-ish response shape for missing and expired codes
structured audit counters without storing sensitive payloads in logs
```

Default limits should assume the public service may be abused as a short-message
store-and-forward relay:

```text
publish code body length: 12 random decimal digits
displayed publish code length: 13 decimal digits including routing prefix
payload cap: 8 KiB
email verification TTL: 30 minutes
default receive TTL after verification: 2 hours
max receive TTL after verification: 2 hours
max receives per publish: 8
per-IP rate limit: 120 requests/minute with burst 40
```

Typed payload validation prevents arbitrary blobs from being stored as published payloads.
The remaining limits bound abuse cost and retention for payloads that are
syntactically valid but still untrusted. A public deployment should still sit
behind normal edge controls, such as TLS termination, connection limits,
firewall rules, and monitoring.

The server should derive `code_hash` and `delete_token_hash` with a
server-secret keyed hash, not a plain unsalted hash:

```text
code_hash = keyed_hash(server_secret, "publish-code" || publish_code)
```

This prevents offline enumeration if segment files are copied.

For QR or link-based sharing, consider embedding a higher-entropy secret in the
link while still displaying a short human publish code for manual entry.

## Runtime Shape

Recommended implementation stack:

```text
tokio
axum
tower-http
bytes
tracing
```

Use structured parsers only for local configuration or diagnostics if needed.
All client/server and server/server communication is binary, versioned, and
handled by explicit codecs.

Keep the HTTP layer thin. Put store behavior behind a trait so purge, replay,
receive, and compaction semantics can be tested and benchmarked without running a
server:

```rust
trait PublishStore {
    fn create(&self, request: CreatePublish) -> Result<CreatedPublish, StoreError>;
    fn receive(&self, code: PublishCode) -> Result<Option<ReceivedPublish>, StoreError>;
    fn delete(&self, code: PublishCode, token: DeleteToken) -> Result<bool, StoreError>;
    fn purge_expired(&self, now: SystemTime) -> Result<usize, StoreError>;
    fn compact(&self) -> Result<CompactionReport, StoreError>;
}
```

Use async methods only where they reflect real async IO. The store can start
with blocking file IO behind `spawn_blocking` or dedicated shard writer tasks.

## Developer Mode

Developer mode should make local testing easy without weakening production
defaults.

Developer mode may enable:

```text
binding to localhost by default
plain HTTP without TLS behind localhost
verbose request tracing
shorter purge intervals
smaller segment size
deterministic publish-code generation for tests
test-only endpoint to dump non-sensitive store stats
```

Developer mode must not log raw payloads, raw publish codes, delete tokens, or
server secrets.

## Self Installation

The server binary should be able to install and remove itself as a systemd
daemon on Linux hosts.

Recommended commands:

```text
revault_key_server install
revault_key_server uninstall
revault_key_server status
revault_key_server run
```

`install` should:

```text
require root or sudo privileges
create a dedicated service user and group
create the config, state, cache, and log directories
install or update the systemd unit file
write a default config file if one does not already exist
generate or load the server secret
run systemctl daemon-reload
enable the service for boot
start or restart the service
```

The service should run as an unprivileged user:

```text
user: lockbox-publish
group: lockbox-publish
```

Default paths:

```text
/etc/lockbox/key-server.toml
/var/lib/lockbox-key-server/
/var/cache/lockbox-key-server/
/var/log/lockbox-key-server/
```

The binary should not require a separate package manager script to be usable.
Packaging can still wrap the same install logic later, but the standalone
binary should support direct installation on a server.

Example systemd unit:

```text
[Unit]
Description=reVault Key Rendezvous Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=lockbox-publish
Group=lockbox-publish
ExecStart=/usr/local/bin/revault_key_server run \
  --config /etc/lockbox/key-server.toml
Restart=always
RestartSec=2
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
PrivateDevices=true
RestrictSUIDSGID=true
LockPersonality=true
MemoryDenyWriteExecute=true
ReadWritePaths=/var/lib/lockbox-key-server \
  /var/cache/lockbox-key-server \
  /var/log/lockbox-key-server
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
```

The install command should preserve existing configuration and secrets. It may
replace the unit file when the generated unit changes, but it must not overwrite
`/etc/lockbox/key-server.toml` unless an explicit `--force-config` option is
provided.

`uninstall` should stop and disable the service, remove the systemd unit, and
run `systemctl daemon-reload`. It should not delete persisted publish data,
server secrets, logs, or config unless passed an explicit destructive option
such as `--purge-data`.

`status` should report:

```text
unit installed
unit enabled
unit active
config path
state path
binary path used by the unit
```

Developer mode may support a user-level systemd install later, but the first
production target is a system service that starts on boot.

## Operational Notes

Expose health and readiness endpoints separately if operational tooling needs
them. These can still use binary POST operations or a minimal plain HTTP path,
depending on deployment requirements.

Health should confirm the process is alive. Readiness should confirm the store
directory is writable, segment replay completed, and purge tasks are running.

Metrics should include:

```text
publishes_created_total
publishes_received_total
publishes_deleted_total
publishes_expired_total
receive_publish_misses_total
rate_limited_total
live_publishes
payload_cache_hit_ratio
segment_bytes_live
segment_bytes_dead
purge_duration
replay_duration
compaction_duration
```

Logs must not include public key payloads, delete tokens, raw publish codes, or
the server secret. Use request IDs and hashed code prefixes for diagnostics.

## Implementation Phases

1. Add crate skeleton and design notes.
2. Add binary envelope codecs and protocol tests.
3. Add config, store trait, and single-shard disk-backed store.
4. Add startup replay, tombstones, and CRC handling.
5. Add expiry buckets and focused purge tests.
6. Add HTTP endpoint, body limits, and binary error responses.
7. Add rate limiting and delete-token validation.
8. Add shard support and concurrent load benchmarks.
9. Add self-install, uninstall, status, and systemd unit generation.
10. Add CLI publish/receive/delete integration.
11. Add compaction and operational metrics.
12. Add topology discovery, standby replication, promotion, and recovery tools.
