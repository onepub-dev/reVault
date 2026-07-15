# Rust toolchain policy

MSRV means Minimum Supported Rust Version: the oldest Rust compiler version this workspace is expected to build with.

The project and public Rust API MSRV is 1.95. This applies to:

- `revault_page_api`
- `revault_lockbox_api`
- `revault_vault_api`
- `bindings/rust` (the published `revault-api` crate)

All workspace crates, applications, migration executables, binding generators,
and published Rust bindings use the same Rust 1.95 baseline.

Development flow:

- Use the pinned Rust 1.95 toolchain when developing locally.
- Run `cargo xtask check-required` before merging substantial Rust changes.
- Run `cargo xtask run-network-tests` when changing publish, topology,
  replication, networking, or key-server behavior.
- Treat an API MSRV increase as an intentional compatibility change. Update the
  affected manifests, this document, CI, and release notes together.

CI flow:

- GitHub Actions verifies the public APIs and complete workspace on Rust 1.95,
  and also exercises the workspace on the latest stable Rust.
- Required checks run formatting, clippy, unit tests, integration tests, and the key-server/publish-protocol crates.
- The network integration job runs ignored loopback tests, including key-server failover and CLI publish/receive flows.
