# Rust toolchain policy

MSRV means Minimum Supported Rust Version: the oldest Rust compiler version this workspace is expected to build with.

The Rust workspace MSRV is 1.88.

Why 1.88:

- `keyring 4.1.3` declares Rust 1.88 as its minimum supported version.
- The vault crate depends on `keyring` for platform credential storage.
- The package manifests set `rust-version = "1.88"` so Cargo reports a clear error on older compilers.

Development flow:

- Use the current stable Rust toolchain when developing locally.
- Run `bash tools/check_required.sh` before merging substantial Rust changes.
- Run `bash tools/run_network_tests.sh` when changing publish, topology, replication, networking, or key-server behavior.
- If a dependency raises its own MSRV above 1.88, either reject that upgrade or update every crate manifest and this document in the same change.

CI flow:

- GitHub Actions installs stable Rust.
- Required checks run formatting, clippy, unit tests, integration tests, and the key-server/publish-protocol crates.
- The network integration job runs ignored loopback tests, including key-server failover and CLI publish/receive flows.
