# Workspace tasks

`xtask` contains repository maintenance and validation commands that are not
part of the shipped reVault applications or libraries.

Run tasks from the `rust/` workspace directory:

```text
cargo xtask help
cargo xtask check-required
cargo xtask install-cli
```

Cargo maps `cargo xtask` to `cargo run -p xtask --` through
`.cargo/config.toml`.

Release and language-binding tooling remains in `revault_tooling` while that
work is being migrated separately.
