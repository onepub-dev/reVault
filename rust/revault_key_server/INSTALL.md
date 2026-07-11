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
