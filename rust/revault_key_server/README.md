# revault_key_server

The reVault key rendezvous server handles high-throughput, short-lived contact
publish and receive operations for the reVault CLI. It relays candidate keys;
it does not establish trust in them.

It relays candidate contact public keys; clients must still verify a received
key's fingerprint independently before trusting it. Publish codes are
self-routing: their first digit identifies the owning server. Servers expose a
topology document that tells clients which primary and failover endpoint serves
each owner id. Standby peers receive signed, idempotent replication events, but
serve replica payloads only after explicit operator promotion. This avoids
hot/hot single-use receive races during a network partition.

Read the full [key-server topology and replication overview](https://github.com/onepub-dev/reVault#key-server-topology-and-replication), [configuration reference](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/KEY_SERVER_CONFIG.md), and [redundancy design](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/REDUNDANCY.md).

Install and manage the Linux system service with:

```bash
cargo install revault_key_server
sudo revault_key_server install
sudo revault_key_server doctor
```

The service uses `/etc/revault/key-server.toml` and stores runtime state under
`/var/lib/revault-key-server`.

Production operation requires the separate commercial license described in the
package license.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.


# Install the reVault key server

This package builds one system binary: `revault_key_server`.

Production operation of this binary requires a separate commercial license from
OnePub IP Pty Ltd. See `LICENSE`.

## 1) Build the binary

```bash
cd /home/bsutton/git/revault/rust
cargo build -p revault_key_server --release
```

## 2) Install as a system service

The release binary is configured as a systemd service named
`revault_key_server.service`.

```bash
sudo ./target/release/revault_key_server install
```

The install command copies the executable to `/usr/local/bin/revault_key_server`
before creating the service. This keeps it accessible with the service's
`ProtectHome=true` hardening even when the installer was run from a user's home
directory.

`--force-config` rewrites `/etc/revault/key-server.toml` only if you want a new
bootstrap config.

```bash
sudo ./target/release/revault_key_server install --force-config
```

## 3) Verify service status

```bash
sudo ./target/release/revault_key_server doctor
```

## 4) Start/stop service manually (if needed)

```bash
sudo systemctl start revault_key_server
sudo systemctl stop revault_key_server
sudo systemctl restart revault_key_server
sudo systemctl status revault_key_server
```

## 5) Remove service

```bash
sudo ./target/release/revault_key_server uninstall
```

Use `--purge-data` only if you also want to remove persisted state, cache,
and config:

```bash
sudo ./target/release/revault_key_server uninstall --purge-data
```

## Notes

- Default config path: `/etc/revault/key-server.toml`
- Default data paths:
  - `/var/lib/revault-key-server`
  - `/var/cache/revault-key-server`
  - `/var/log/revault-key-server`

Existing installations from `0.0.1` used `/etc/lockbox` and
`/var/*/lockbox-key-server`. Review and migrate the old configuration and
state before starting the renamed service; the installer does not silently
move persisted publish data.


## License

See the repository license for licensing terms.
