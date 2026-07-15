# Key Server Configuration

The reVault key server reads its server configuration from a TOML-style config
file. Each process is simultaneously a publish/receive service, a topology
endpoint, and (when configured) a replication peer. There is no separate
topology-server process.

Default path:

```text
/etc/revault/key-server.toml
```

When installed with:

```bash
sudo revault_key_server install
```

the installer creates the config file if it does not already exist. Use
`--force-config` only when you intentionally want to replace the existing
bootstrap config.

Production operation of the key server requires a separate commercial
license.

For architecture and operational context, start with the [server
introduction](README.md), then use this reference while preparing the config.
See [Troubleshooting](TROUBLESHOOTING.md) for runtime diagnosis.

## Default Config

```toml
bind_addr = "0.0.0.0:8089"
state_dir = "/var/lib/revault-key-server"

server_id = 0
cluster_id = "default"
public_url = "https://keyshare0.revault.onepub.dev/v1/publish"

topology_version = 1

origin_epoch = 1

verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
max_payload_bytes = 8192
max_receives_per_publish = 8

rate_limit_per_minute = 120
rate_limit_burst = 40

smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = ""
smtp_password = ""
smtp_from = ""
smtp_tls = "starttls"
smtp_timeout_seconds = 30
verification_email_subject = "Verify your reVault publish"
verification_email_template = "Verify {email} for this reVault publish:\n\n{verification_url}\n\nThis link expires in 30 minutes."
verification_email_rate_limit_per_hour = 5
verification_email_ip_rate_limit_per_hour = 30

[[topology_server]]
id = 0
url = "https://keyshare0.revault.onepub.dev/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = []
```

## File Format

The config file uses TOML-style scalar `key = value` settings plus arrays of
tables for topology members and routes.

Comments are supported:

```toml
# Listen on all interfaces
bind_addr = "0.0.0.0:8089"
```

String values may be quoted:

```toml
cluster_id = "production"
```

Numeric and boolean values do not need quotes:

```toml
server_id = 0
developer_mode = false
```

Some scalar keys may be repeated to create lists:

```toml
replication_peer_url = "https://keyshare1.example.com/v1/replicate"
replication_peer_url = "https://keyshare2.example.com/v1/replicate"
```

Topology members and routes use TOML arrays of tables:

```toml
[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keyshare1.example.com/v1/publish"
status = "standby"

[[route]]
owner = 0
primary = 0
failover = [1]
```

Unknown keys are rejected.

## Core Server Settings

| Key | Default | Description |
| --- | --- | --- |
| `bind_addr` | `127.0.0.1:8089` | Local address and port the HTTP server binds to. The installed config uses `0.0.0.0:8089`. |
| `state_dir` | `/var/lib/revault-key-server` | Directory used for persisted publish records, indexes, replication state, and server secret material. |
| `server_id` | `0` | Stable routing id for this server. Valid ids are `0..35`, written as `0..9`, `a..z`, or numeric values. |
| `cluster_id` | `default` | Public cluster identifier returned in topology documents. All cooperating servers should use the same cluster id. |
| `public_url` | derived from `bind_addr` | External URL for this server's publish API. Clients and peer servers use this URL for publish, receive, delete, and replication routing, so set it to the reachable HTTPS URL in production. |
| `developer_mode` | `false` | Developer/test mode. Do not enable in production. When enabled, the server uses a temporary state directory. |

## Publish Limits

TTL settings use seconds in the config file:

- `verification_ttl_seconds` controls how long the email verification link is valid.
- `default_receive_ttl_seconds` controls the default receive window after email verification.
- `max_receive_ttl_seconds` caps a client-requested receive TTL after email verification.

| Key | Default | Description |
| --- | --- | --- |
| `verification_ttl_seconds` | `1800` | Email verification link lifetime. A pending publish cannot be received until the email is verified. |
| `default_receive_ttl_seconds` | `7200` | Default receive lifetime after email verification when the client does not request a TTL. |
| `max_receive_ttl_seconds` | `7200` | Maximum receive lifetime after email verification. Client-requested TTLs are capped to this value. |
| `max_payload_bytes` | `8192` | Maximum encoded publish payload size accepted by the server. |
| `max_receives_per_publish` | `8` | Maximum successful receives allowed for one published payload. A requested value above this is capped. |

## Rate Limits

| Key | Default | Description |
| --- | --- | --- |
| `rate_limit_per_minute` | `120` | Per-IP request rate limit. Use `0` to disable. Unauthenticated `GET /v1/topology`, `GET /v1/status`, `GET /v1/verify`, and non-tokened topology registration or replication requests use this limiter. |
| `rate_limit_burst` | `40` | Per-IP burst capacity. |

## Email Verification

The key server requires a publisher email address for every publish. It sends an
email verification link and keeps the publish pending until the link is used.
Receive attempts before verification fail with `EmailUnverified`, so the
receiver can tell the publisher what is blocking the receive.

| Key | Default | Description |
| --- | --- | --- |
| `smtp_host` | empty | SMTP server hostname. The installed template uses `smtp.gmail.com` for Gmail. |
| `smtp_port` | `587` | SMTP port. Gmail STARTTLS uses `587`; implicit TLS commonly uses `465`. |
| `smtp_username` | empty | SMTP username. For Gmail, use the Gmail address or configured account username. |
| `smtp_password` | empty | SMTP password. For Gmail accounts with 2-step verification, use an app password. |
| `smtp_from` | empty | Sender email address. If empty, `smtp_username` is used. |
| `smtp_tls` | `starttls` | SMTP TLS mode: `starttls`, `tls`, or `none`. |
| `smtp_timeout_seconds` | `30` | SMTP send timeout used by the bounded background email worker. |
| `verification_email_subject` | `Verify your reVault publish` | Subject template. Placeholders: `{email}`, `{publish_code}`, `{verification_url}`. |
| `verification_email_template` | see default config | Plain text body template. Placeholders: `{email}`, `{publish_code}`, `{verification_url}`. Use `\n` for newlines. |
| `verification_email_rate_limit_per_hour` | `5` | Maximum verification emails per email address per hour across the cluster. Use `0` to disable this limit. |
| `verification_email_ip_rate_limit_per_hour` | `30` | Maximum verification emails per source IP per hour. Use `0` to disable this limit. |

Example:

```toml
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "publisher@example.com"
smtp_password = "gmail-app-password"
smtp_from = "publisher@example.com"
smtp_tls = "starttls"
smtp_timeout_seconds = 30
verification_email_subject = "Verify your reVault publish"
verification_email_template = "Verify {email} for publish {publish_code}:\n\n{verification_url}\n\nThis link expires in 30 minutes."
verification_email_rate_limit_per_hour = 5
verification_email_ip_rate_limit_per_hour = 30
```

Publish requests queue verification email work on a bounded background worker.
A full or unavailable queue causes the publish request to fail fast; SMTP
delivery failures after queuing are logged.

For clustered deployments, the topology deterministically assigns each
normalized publisher email address to one primary server plus one backup server.
Only those two servers may queue verification email for the address, and
topology-aware clients route the publish to the primary first. Direct publish
attempts to other servers are rejected as rate limited, which prevents an
attacker from multiplying the per-email limit by calling every public server.

Topology-aware clients fail over verification-email publishes only to that one
backup server, and only when the primary is unavailable. If both are unavailable,
the client reports that the verification email service is temporarily
unavailable and asks the publisher to try again shortly.

Verification email or source-IP rate-limit breaches create temporary local
blocks so repeated attempts are rejected before payload validation and before
SMTP queueing. Source-IP verification blocks are also replicated to configured
peers.

## Topology Settings

The key server exposes public routing metadata through its topology endpoint.
Clients use this to route publish, receive, and delete operations to the correct
server.

| Key | Default | Description |
| --- | --- | --- |
| `topology_version` | `1` | Public topology version. Increase when making manual topology changes. |
| `[[topology_server]]` | none | Adds a key-server member to the public topology. Its URL is the member's `/v1/publish` URL. |
| `[[route]]` | auto-generated | Adds an owner routing rule. |
| `topology_token` | none | Shared token used in the `X-Lockbox-Server-Token` or `Authorization: Bearer` header for topology heartbeat registration and unmetered peer topology/status/replication requests. Do not put this token in public topology URLs. |
| `topology_stale_after_ms` | `90000` | Ignore topology peers that have not checked in within this age. |
| `topology_heartbeat_interval_ms` | `30000` | Interval between topology heartbeat posts. |

### `topology_server`

Despite the historical setting name, this is not a topology-only server. It is
a key-server cluster member advertised by every member's `/v1/topology`
endpoint. Always configure its public `/v1/publish` URL; peers derive
`/v1/topology/register` and clients derive `/v1/topology` from the same origin.

Format:

```toml
[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"
```

Valid statuses:

```text
active
standby
promoted
disabled
```

If `status` is omitted, `active` is used.

Example:

```toml
[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keyshare1.example.com/v1/publish"
status = "standby"
```

### `route`

Format:

```toml
[[route]]
owner = 0
primary = 0
failover = [1]
```

Example:

```toml
[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]
```

This means:

- published payloads owned by server id `0` are served by server `0`, with server `1` as
  failover
- published payloads owned by server id `1` are served by server `1`, with server `0` as
  failover

If no explicit routes are configured, the server builds ring routes from the
configured key-server members. Explicit routes are recommended in production
because they make authority and promotion intent reviewable.

## Replication Settings

Replication is used to copy publish state to peer servers. Replicated published payloads are
not served by a standby unless that standby is explicitly promoted for the owner
id.

| Key | Default | Description |
| --- | --- | --- |
| `replication_token` | none | Shared secret used to sign and verify peer replication messages. Required for replication and `resync-peer`. Peer replication HTTP requests should also include the `topology_token` header so they are treated as trusted inter-server traffic and bypass the public IP limiter. |
| `replication_peer_url` | none | Peer `/v1/replicate` endpoint. May be repeated. |
| `origin_epoch` | current time in milliseconds | Local replication epoch used for conflict/idempotency tracking. Installed bootstrap config sets this to `1`. |
| `promoted_owner` | none | Owner id this server is allowed to serve as a promoted standby. May be repeated. |

Example two-server replication pair:

```toml
# server 0
server_id = 0
replication_token = "replace-with-a-long-random-secret"
replication_peer_url = "https://keyshare1.example.com/v1/replicate"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]
```

```toml
# server 1
server_id = 1
replication_token = "replace-with-a-long-random-secret"
replication_peer_url = "https://keyshare0.example.com/v1/replicate"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]
```

To promote server `1` to serve published payloads owned by server `0`:

```toml
server_id = 1
promoted_owner = 0
```

Only promote a standby when the original owner is no longer serving that owner
id.

## Storage Settings

| Key | Default | Description |
| --- | --- | --- |
| `shard_count` | `16` | Number of local store shards. Usually leave unchanged after deployment. |
| `index_cache_entries` | `65536` | Maximum cached index entries. |
| `compact_min_bytes` | `67108864` | Segment size threshold before background compaction is considered. |

## Single Server Example

```toml
bind_addr = "0.0.0.0:8089"
state_dir = "/var/lib/revault-key-server"

server_id = 0
cluster_id = "production"
public_url = "https://keyshare.example.com/v1/publish"

topology_version = 1

verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
max_payload_bytes = 8192
max_receives_per_publish = 8

rate_limit_per_minute = 120
rate_limit_burst = 40

smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "publisher@example.com"
smtp_password = "gmail-app-password"
smtp_from = "publisher@example.com"
smtp_tls = "starttls"
verification_email_subject = "Verify your reVault publish"
verification_email_template = "Verify {email} for this reVault publish:\n\n{verification_url}\n\nThis link expires in 30 minutes."
verification_email_rate_limit_per_hour = 5
verification_email_ip_rate_limit_per_hour = 30

[[topology_server]]
id = 0
url = "https://keyshare.example.com/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = []
```

## Two Server Example

Both nodes use the same `cluster_id`, member list, routes,
`topology_token`, and `replication_token`. Each node has its own `server_id`,
`public_url`, state directory, and peer replication URL. The examples use
placeholders; store long random tokens in the root-readable configuration and
never place them in public URLs.

Server 0:

```toml
bind_addr = "0.0.0.0:8089"
state_dir = "/var/lib/revault-key-server"

server_id = 0
cluster_id = "production"
public_url = "https://keyshare0.example.com/v1/publish"

topology_version = 1
topology_token = "replace-with-a-long-random-topology-secret"

[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keyshare1.example.com/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]

replication_token = "replace-with-a-long-random-secret"
replication_peer_url = "https://keyshare1.example.com/v1/replicate"

verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
max_payload_bytes = 8192
max_receives_per_publish = 8
```

Server 1:

```toml
bind_addr = "0.0.0.0:8089"
state_dir = "/var/lib/revault-key-server"

server_id = 1
cluster_id = "production"
public_url = "https://keyshare1.example.com/v1/publish"

topology_version = 1
topology_token = "replace-with-a-long-random-topology-secret"

[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keyshare1.example.com/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]

replication_token = "replace-with-a-long-random-secret"
replication_peer_url = "https://keyshare0.example.com/v1/replicate"

verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
max_payload_bytes = 8192
max_receives_per_publish = 8
```

## Operational Notes

- Keep `server_id` stable. A replacement machine serving the same owner id must
  use the same `server_id`.
- Set `public_url` in production. Do not rely on the bind address being
  publicly meaningful.
- Protect the config file if it contains `replication_token`.
- Do not enable `developer_mode` in production.
- Do not use DNS round-robin as the only failover mechanism. Clients use
  topology and the published payload-code routing id to find the correct owner server.
- The server stores durable state under `state_dir`; back it up according to
  your operational recovery requirements.
