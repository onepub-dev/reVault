# revault_key_server

High-throughput reVault key rendezvous server for short-lived contact publish
and receive operations.

Install and manage the Linux system service with:

```bash
cargo install revault_key_server
sudo revault_key_server install
sudo revault_key_server doctor
```

The service uses `/etc/revault/key-server.toml` and stores runtime state under
`/var/lib/revault-key-server`. See `INSTALL.md` and `KEY_SERVER_CONFIG.md` for
deployment and configuration details.

Production operation requires the separate commercial license described in the
package license.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

See the repository license for licensing terms.
