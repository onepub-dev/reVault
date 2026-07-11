# revault_key_server

`revault_key_server` helps two reVault users begin sharing lockboxes without
having to send public keys manually through email or chat.

## Where it fits

In reVault:

- A **lockbox** is an encrypted `.lbox` archive containing files, variables,
  or form records.
- A **vault** is a user's local private store for identities, contacts, and
  the keys needed to open lockboxes.
- An **identity** is the user's public/private key pair.
- A **contact** is another person's verified public identity key.
- This **key server** is a temporary meeting point that lets one user offer a
  candidate public key to another user.

The key server is not a vault and does not hold private keys. It cannot decide
who somebody is or make a candidate key trusted. Those decisions stay on the
users' devices.

## What happens when users connect

1. Alice asks the reVault CLI to publish her identity's public key.
2. The server sends Alice an email verification link, so it does not make an
   unverified address immediately available.
3. Once verified, the server returns a short-lived **publish code**.
4. Alice gives that code to Bob through a separate channel—for example a phone
   call, chat, or in person.
5. Bob receives Alice's candidate public key with the code, then compares its
   **fingerprint** with Alice over an independent channel before saving her as a
   trusted contact.

The code is for finding a candidate key; the fingerprint check is what creates
trust. A publish expires and can be consumed only a limited number of times.

## One server or several

Start with one server. If you later need redundancy, each server has a stable
numeric id. That id is the first digit of a publish code, so clients can route
receive and delete requests to the server that owns the code.

Servers share a **topology** document: a small map of server ids, URLs, and
primary/failover routes. A client publishes through one selected server; for a
receive it reads the code's first digit, contacts the owning primary server,
and then tries only that server's configured failovers.

Standby servers receive signed replication events for pending publishes,
deletions, receive counts, and abuse blocks. A standby stores the copy but does
not serve it until an operator explicitly promotes that standby for the failed
server id. This design prevents two servers from consuming the same one-time
publish during a network partition.

Read the full [key-server topology and replication overview](https://github.com/onepub-dev/reVault#key-server-topology-and-replication), [configuration reference](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/KEY_SERVER_CONFIG.md), and [redundancy design](https://github.com/onepub-dev/reVault/blob/master/rust/revault_key_server/REDUNDANCY.md).

## Install and operate

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


## Build from the repository

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
