# Deprecated historical migration package

Use [revault_migrate_v2](https://crates.io/crates/revault_migrate_v2) through `lockbox doctor migrate` instead.

Historical readers are now bundled privately inside one exporter package per archive/container version. This package is retained for reproducible historical dependencies and regression fixtures; it is excluded from normal releases. Existing published versions remain available and are not yanked. Archive/container version and Vault structure version are separate: container 1 can hold Vault structures 1 or 2.

---

# Vault structure 2 migration exporter

This frozen exporter reads vault structure 2 in lockbox container format 2 using
published `revault_vault_api = 0.0.10` and `revault_lockbox_api = 0.0.9` readers.
It emits encrypted migration schema 2 records for the current CLI to upgrade
and import. It does not import vaults or understand password profiles.

Install alongside the current CLI:

```console
cargo install revault_migrate_vault_v2 --version 0.0.1
lbx doctor migrate vault --replace
```

The CLI can install the registered exporter automatically. For a locally built
exporter, pass `--exporter /path/to/revault-migrate-vault-v2`.

The exporter receives passwords through the framed stdin migration protocol.
`revault-migrate-vault-v2 capabilities` reports its supported versions.
