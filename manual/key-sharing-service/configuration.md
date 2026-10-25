---
description: "Configure and operate a reVault key-sharing server."
---

# Key-server configuration

Each `revault_key_server` process provides publishing, receiving, topology and optional replication endpoints. There is no separate topology-server program.

Production operation requires a separate commercial licence.

## Install the service

```bash
cargo install revault_key_server
sudo revault_key_server install
sudo revault_key_server doctor
```

The Linux installer creates:

| Purpose | Path |
| --- | --- |
| Executable | `/usr/local/bin/revault_key_server` |
| Configuration | `/etc/revault/key-server.toml` |
| State | `/var/lib/revault-key-server` |
| Log | `/var/log/revault-key-server/server.log` |
| systemd unit | `/etc/systemd/system/revault_key_server.service` |

`install` preserves an existing configuration. `install --force-config` deliberately replaces it with the bootstrap template.

{% hint style="danger" %}
Keep the configuration readable only by the service account and administrators. It may contain SMTP, topology and replication credentials.
{% endhint %}

## Bootstrap configuration

The installed template listens on loopback so that a production TLS reverse proxy can expose it deliberately:

```toml
bind_addr = "127.0.0.1:8089"
state_dir = "/var/lib/revault-key-server"

server_id = 0
cluster_id = "default"
public_url = "https://keyshare0.example.com/v1/publish"
topology_version = 1
origin_epoch = 1

verification_ttl_seconds = 1800
default_receive_ttl_seconds = 7200
max_receive_ttl_seconds = 7200
max_payload_bytes = 8192
max_receives_per_publish = 8

rate_limit_per_minute = 120
rate_limit_burst = 40

smtp_host = "smtp.example.com"
smtp_port = 587
smtp_username = "publisher@example.com"
smtp_password = "replace-me"
smtp_from = "publisher@example.com"
smtp_tls = "starttls"
smtp_timeout_seconds = 30

verification_email_subject = "Verify your reVault publish"
verification_email_template = "Verify {email} for this reVault publish:\n\n{verification_url}\n\nThis link expires in 30 minutes."
verification_email_rate_limit_per_hour = 5
verification_email_ip_rate_limit_per_hour = 30

[[topology_server]]
id = 0
url = "https://keyshare0.example.com/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = []
```

The public member URL is always the `/v1/publish` endpoint. Peers and clients derive `/v1/topology`, `/v1/topology/register` and the other endpoints from the same origin.

## Native HTTPS and Let's Encrypt

Set these top-level keys before any `[[topology_server]]` or `[[route]]` tables:

```toml
bind_addr = "[2001:db8::2]:443"
acme_domain = "keyshare0.example.com"
acme_email = "operator@example.com"
acme_cache_dir = "/var/lib/revault-key-server/acme"
acme_production = false
acme_http_port = 80
```

Replace the documentation address with the server's allocated, persistent IPv6 address.
A nonempty `acme_domain` enables native HTTPS on `bind_addr`; an empty domain
retains plain HTTP for local development or an existing reverse proxy.
The HTTPS bind must specify a single IP address, not a wildcard.

The server obtains and renews certificates with HTTP-01 validation. Port 80 on the
same IP serves only `/.well-known/acme-challenge/{token}`; key APIs remain HTTPS-only.
Permit inbound TCP ports 80 and 443 and outbound HTTPS to the ACME CA. Point DNS at
this address. If Cloudflare proxies the hostname, allow the HTTP validation path
through and ensure its edge certificate covers the full hostname. Use Full (strict)
with a production origin certificate.

Start with `acme_production = false` to validate issuance against Let's Encrypt
staging, then set it to `true` and restart for a publicly trusted certificate.
Staging certificates are not browser-trusted. The server keeps separate staging
and production subdirectories under `acme_cache_dir`, with mode 0700 on Unix.
Preserve that directory across upgrades: it contains private account and certificate
keys. Renewal retries occur in the running process, and renewed certificates are
used for new TLS connections without restarting.

The Linux systemd unit grants only `CAP_NET_BIND_SERVICE` for ports 80/443 and uses
a private umask. Existing installations need the updated unit (rerun `install`,
which preserves configuration), followed by a service restart. Keep a custom
cache directory inside a service-writable path, or explicitly adjust the unit's
`ReadWritePaths`.

## Core settings

| Key | Runtime default | Purpose |
| --- | --- | --- |
| `bind_addr` | `127.0.0.1:8089` | Local listening address. |
| `state_dir` | `/var/lib/revault-key-server` | Persistent publish, index, replication and server-secret state. |
| `server_id` | `0` | Stable member/owner identifier from `0` to `35`. |
| `cluster_id` | `default` | Cluster name; identical on cooperating members. |
| `public_url` | derived locally | Reachable HTTPS `/v1/publish` URL. Set this explicitly in production. |
| `developer_mode` | `false` | Enables temporary developer behaviour. Never enable it in production. |

Keep `server_id` stable when replacing a machine. Set `origin_epoch` to a stable deployment value; changing it affects replication conflict and idempotency tracking.

## Publish and email limits

| Key | Default | Purpose |
| --- | --- | --- |
| `verification_ttl_seconds` | `1800` | Lifetime of the email-verification link. |
| `default_receive_ttl_seconds` | `7200` | Default receive window after verification. |
| `max_receive_ttl_seconds` | `7200` | Maximum client-requested receive window. |
| `max_payload_bytes` | `8192` | Maximum encoded public Profile payload. |
| `max_receives_per_publish` | `8` | Successful receives allowed per publish. |
| `rate_limit_per_minute` | `0` | Token refill rate per minute for each socket peer; `0` disables it. The example above sets `120`. |
| `rate_limit_burst` | `0` | Token capacity for each socket peer; the example above sets `40`. With limiting enabled, `0` allows one request at a time. |

The server applies these limits to public publish, receive, delete, topology, status and verification requests using the TCP peer address. When it runs behind a reverse proxy, that address is the proxy, so all clients share one server bucket. Configure per-client limits in the trusted proxy as well. The server ignores `X-Forwarded-For` and similar headers supplied by clients.

Email settings are `smtp_host`, `smtp_port`, `smtp_username`, `smtp_password`, `smtp_from`, `smtp_tls` and `smtp_timeout_seconds`. Valid TLS modes are `starttls`, `tls` and `none`; do not use `none` across an untrusted network.

The subject and body support `{email}`, `{publish_code}` and `{verification_url}`. Configure them with `verification_email_subject` and `verification_email_template`. The per-address and per-socket-peer hourly controls are `verification_email_rate_limit_per_hour` and `verification_email_ip_rate_limit_per_hour`; `0` disables the corresponding limit. The verification URL is sent only to the mailbox and is never returned by the publish API.

## Topology and replication

The topology settings are:

* `topology_version`
* `topology_token`
* `topology_stale_after_ms` (default `90000`)
* `topology_heartbeat_interval_ms` (default `30000`)
* one or more `[[topology_server]]` tables
* optional `[[route]]` tables

Read [Topology and failover](../topology-server.md) before configuring more than one member.

Replication uses `replication_token` and one or more `replication_peer_url` values pointing to `/v1/replicate`. `promoted_owner` authorises a standby to serve a failed owner's publishes. It may be repeated. Promotion is an explicit operator action.

```toml
topology_token = "replace-with-a-long-random-topology-secret"
replication_token = "replace-with-a-different-long-random-secret"
replication_peer_url = "https://keyshare1.example.com/v1/replicate"
promoted_owner = 0
```

Use `revault_key_server resync-peer --peer-url https://keyshare1.example.com/v1/replicate` after a peer has missed replication state.

## Storage settings

| Key | Default | Purpose |
| --- | --- | --- |
| `shard_count` | `16` | Number of local store shards. |
| `index_cache_entries` | `65536` | Maximum cached index entries. |
| `compact_min_bytes` | `67108864` | Segment threshold before background compaction. |

Choose these before production deployment and change them only with a tested migration plan.

## Operate and diagnose

```bash
sudo revault_key_server doctor
sudo revault_key_server stop
sudo revault_key_server start
sudo journalctl -u revault_key_server -n 50 --no-pager
```

Run the exact installed version's `--help` before using developer or recovery options. Unknown configuration keys are rejected, and `doctor` reports invalid configuration, incomplete SMTP settings, legacy paths and service-account permission problems.
