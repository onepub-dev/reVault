# reVault key server

`revault_key_server` is the temporary rendezvous service used by reVault users
to exchange candidate public profile keys. It stores short-lived published
payloads, sends email-verification links, and returns self-routing publish
codes. It never stores private profile keys and does not decide whether a
candidate key is trusted; recipients establish trust by checking its
fingerprint over an independent channel.

## How the service fits together

A production node performs three related jobs:

1. **Key service:** accepts publish, receive, and delete requests.
2. **Topology service:** exposes `GET /v1/topology`, describing every key-server
   member and the primary/failover route for each publish-code owner id.
3. **Replication peer:** sends signed state events to a standby so that an
   operator can promote it when an owner server fails.

There is no separate topology-server binary. Every configured key server serves
the topology endpoint. The `[[topology_server]]` configuration name means “a
key-server member advertised in topology,” not a topology-only machine.

Publish codes begin with the stable owner server id. Clients discover the
cluster through one of the public topology endpoints, publish through the
server selected for the verified email, and route receive/delete operations by
the code's owner id. The CLI's production bootstrap endpoints are:

```text
https://keyshare0.revault.onepub.dev/v1/topology
https://keyshare1.revault.onepub.dev/v1/topology
```

It tries them in order for initial discovery. Once discovery succeeds, the
topology document supplies the complete current server and failover list.

## Documentation map

- [Design](DESIGN.md): protocol, security, storage, and service architecture.
- [Redundancy design](REDUNDANCY.md): ownership, topology, replication,
  promotion, and recovery semantics.
- [Configuration guide](KEY_SERVER_CONFIG.md): complete settings reference and
  single-node/two-node examples.
- [Troubleshooting](TROUBLESHOOTING.md): DNS, TLS, topology, SMTP, replication,
  routing, and service diagnostics.
- [Command reference](CLI_SWITCHES.md): server commands and developer options.
- [Client publish design](CLI_VAULT_PUBLISH_DESIGN.md): profile/contact CLI
  protocol and historical implementation decisions.
- [Benchmarks](BENCHMARKS.md): load model, measurements, and performance notes.

## Install

Production operation requires the separate commercial license described in
[LICENSE](LICENSE).

Install from crates.io:

```bash
cargo install revault_key_server
sudo revault_key_server install
```

Or build and install from this repository:

```bash
cd rust
cargo build -p revault_key_server --release
sudo ./target/release/revault_key_server install
```

Installation creates:

- service: `revault_key_server.service`
- executable: `/usr/local/bin/revault_key_server`
- configuration: `/etc/revault/key-server.toml`
- state: `/var/lib/revault-key-server`
- cache: `/var/cache/revault-key-server`
- logs: `/var/log/revault-key-server`

The installer preserves an existing configuration. Use `--force-config` only
when you deliberately want to replace it with the bootstrap template.

## Configure

For a production node, at minimum set:

- a unique, stable `server_id`;
- the same `cluster_id` on all members;
- this node's public `https://.../v1/publish` URL;
- every cluster member in `[[topology_server]]`;
- owner routes in `[[route]]`;
- the same `topology_token` on all members for authenticated heartbeats;
- replication peer URLs and a shared `replication_token`;
- complete SMTP credentials.

The public OnePub deployment uses `keyshare<n>.revault.onepub.dev`. A two-node
member list begins as follows:

```toml
server_id = 0
cluster_id = "revault-production"
public_url = "https://keyshare0.revault.onepub.dev/v1/publish"

[[topology_server]]
id = 0
url = "https://keyshare0.revault.onepub.dev/v1/publish"
status = "active"

[[topology_server]]
id = 1
url = "https://keyshare1.revault.onepub.dev/v1/publish"
status = "active"

[[route]]
owner = 0
primary = 0
failover = [1]

[[route]]
owner = 1
primary = 1
failover = [0]
```

See the [configuration guide](KEY_SERVER_CONFIG.md) before starting the service;
the example above intentionally omits secrets, SMTP, storage, and replication
settings.

## Verify and operate

```bash
sudo revault_key_server doctor
sudo systemctl restart revault_key_server
sudo systemctl status revault_key_server
sudo journalctl -u revault_key_server -n 100 --no-pager
```

Run `doctor` after every configuration or deployment change. It validates the
configuration and reports service, state-directory, topology, and SMTP
readiness. Then verify `/v1/topology` on every public member as described in
[Troubleshooting](TROUBLESHOOTING.md).

To remove only the service and installed binary:

```bash
sudo revault_key_server uninstall
```

`uninstall --purge-data` also removes persisted state, cache, and configuration
and should be used only when that destruction is intended.

Existing `0.0.1` installations used `/etc/lockbox` and
`/var/*/lockbox-key-server`. Migrate that configuration and state explicitly;
the installer does not silently move pending publishes.
