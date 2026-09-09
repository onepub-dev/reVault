# Workspace tasks

`xtask` contains repository maintenance and validation commands that are not
part of the shipped reVault applications or libraries.

Run tasks from the `rust/` workspace directory:

```text
cargo xtask help
cargo xtask check-required
cargo xtask install-cli
cargo xtask upgrade-deps
```

`upgrade-deps` uses `cargo upgrade` from `cargo-edit` with incompatible
versions enabled, so it can widen dependency requirements across major
releases. It processes the main workspace, fuzz workspace, Rust bindings, and
Rust binding conformance workspace. It preserves and checks the Rust 1.88
toolchain policy, then runs Clippy, tests, and `cargo tree` for each workspace.
Tests are run with one test thread so CLI session-agent and vault-locking tests
do not interfere with one another.

If `cargo-upgrade` is not already available, `upgrade-deps` installs the
Rust-1.88-compatible `cargo-edit` 0.13.7 release automatically.

Cargo maps `cargo xtask` to `cargo run -p xtask --` through
`.cargo/config.toml`.

Release and language-binding tooling remains in `revault_tooling` while that
work is being migrated separately.
