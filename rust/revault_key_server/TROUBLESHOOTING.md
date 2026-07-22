# Key server troubleshooting

This guide starts at the client-visible symptom and works inward through DNS,
TLS, topology, routing, SMTP, replication, and local service state.

## First checks

On each key server, run:

```bash
sudo revault_key_server doctor
sudo systemctl status revault_key_server --no-pager
sudo journalctl -u revault_key_server -n 100 --no-pager
```

`doctor` should report a valid configuration, the expected public URL, a
writable state directory, at least one topology member and route, and complete
SMTP configuration. Treat warnings as configuration failures for production.

After editing `/etc/revault/key-server.toml`, restart and repeat the checks:

```bash
sudo systemctl restart revault_key_server
sudo revault_key_server doctor
```

## Publish fails with a DNS lookup error

An error such as:

```text
http error: io: failed to lookup address information: Name or service not known
```

means the client could not resolve the hostname; it has not reached HTTP, TLS,
topology decoding, or the publish handler.

The default CLI first tries these topology endpoints:

```text
keyshare0.revault.onepub.dev
keyshare1.revault.onepub.dev
```

Check both records from the affected client network:

```bash
getent hosts keyshare0.revault.onepub.dev
getent hosts keyshare1.revault.onepub.dev
```

If both fail, correct the public DNS records or the client's resolver/network.
If only one fails, initial discovery should use the other; confirm that the CLI
is current and that neither `--topology-url` nor local configuration overrides
the defaults.

Useful one-off isolation commands are:

```bash
lbx vault profile publish --topology-url https://keyshare0.revault.onepub.dev/v1/topology
lbx vault profile publish --topology-url https://keyshare1.revault.onepub.dev/v1/topology
```

Use `--server` only to test a known publish endpoint. It bypasses initial
topology discovery and should not become the normal production configuration.

## TLS or connection failure

Confirm that the public origin accepts HTTPS and serves the intended
certificate:

```bash
curl --fail --show-error --silent \
  https://keyshare0.revault.onepub.dev/v1/topology --output /tmp/topology.bin
```

A certificate-name error usually means the reverse proxy is serving the wrong
virtual host or certificate. A refusal/timeout means DNS succeeded but no
reachable listener or proxy route exists. Check firewall rules, proxy upstream
configuration, the server bind address, and service status.

The topology response is a binary `LBST` document, not JSON or HTML. Receiving
an HTML login/error page indicates a proxy route or authentication mistake.

## Topology discovery fails

Every public member must expose `/v1/topology` without requiring a client
token. Compare the configuration on all members:

- identical `cluster_id`;
- unique stable `server_id` values;
- correct public `/v1/publish` URLs in every `[[topology_server]]` entry;
- consistent routes;
- matching `topology_token` values for peer heartbeat registration;
- clocks close enough for stale-member handling to be meaningful.

The `[[topology_server]]` URL is a publish URL. The server derives the topology
registration endpoint from its origin. Do not configure it as `/v1/topology`.

If a member disappears after roughly `topology_stale_after_ms`, inspect peer
logs for rejected heartbeat registration, cluster-id mismatch, bad token, or an
unreachable `/v1/topology/register` endpoint.

## Publish reaches the wrong server or is rate limited

For email-verified publishing, topology deterministically assigns an email to
one primary and one backup. A direct request to another member can be rejected.
Verify that the client used topology discovery and that every server advertises
the same topology version, member set, and routes.

Do not work around a `RateLimited` response by trying another server. The
client deliberately stops failover on rate limiting so that hopping cannot
bypass abuse controls. Wait for the block/window to expire and investigate
repeated client requests if the limit is unexpected.

## Receive or delete fails for a valid code

The first publish-code character identifies its owner. The topology route for
that owner must point to the authoritative server or to a deliberately promoted
standby holding replicated state.

Check:

- the route contains the code's owner id;
- the primary server is reachable;
- the publish has not expired, been deleted, or exhausted its receive count;
- replication was healthy before promoting a standby;
- only one server is authoritative for the owner id.

Do not promote two servers for the same owner. Single-use receive semantics are
not safe under dual authority.

## Verification email is not delivered

`doctor` must report `SMTP complete: YES`. Confirm `smtp_host`, `smtp_port`,
`smtp_username`, `smtp_password`, `smtp_from`, and `smtp_tls` in the protected
configuration file. Then inspect the journal for connection, authentication,
TLS, queue, and provider rejection errors.

The server distinguishes a queue/send outage from rate limiting. Repeatedly
retrying can create or extend abuse blocks, so fix SMTP connectivity or provider
credentials before publishing again.

## Replication is unhealthy

Each peer needs:

- the same `replication_token` for signed envelopes;
- the peer's public `/v1/replicate` URL in `replication_peer_url`;
- the shared `topology_token` for trusted inter-server HTTP traffic;
- stable `server_id` and `origin_epoch` values;
- writable durable state and outbound outbox storage.

Inspect `/v1/status` and the service logs for sequence gaps, rejected
signatures, token failures, unreachable peers, or growing lag. Do not promote a
standby until its replicated state is sufficiently current for the affected
owner. Use the documented `resync-peer` workflow before handback or when
sequence continuity must be restored.

## Sticky client selection looks stale

The CLI persists its selected publish member in the vault directory as
`.publish-server-sticky`. The selection expires automatically. A topology
update also clears it if that member no longer exists.

Do not delete this file as the first response to a network failure: initial
topology discovery and member reachability should be fixed instead. Remove it
only when deliberately resetting a demonstrably stale local selection.

## Escalation information

When reporting a problem, include:

- exact client command and error, with secrets removed;
- CLI and server versions;
- `doctor` output;
- relevant `journalctl` lines and timestamps;
- which topology FQDNs resolve and accept TLS;
- server id, cluster id, topology version, and advertised routes;
- whether the failure affects publish, receive, delete, verification, or
  replication.

Never include SMTP passwords, topology/replication tokens, delete tokens,
private profile keys, or complete sensitive payloads.
