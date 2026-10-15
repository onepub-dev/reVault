# Deprecated historical migration package

Use [revault_migrate_v1](https://crates.io/crates/revault_migrate_v1) through `lockbox doctor migrate` instead.

Historical readers are now bundled privately inside one exporter package per archive/container version. This package is retained for reproducible historical dependencies and regression fixtures; it is excluded from normal releases. Existing published versions remain available and are not yanked. Archive/container version and Vault structure version are separate: container 1 can hold Vault structures 1 or 2.

---

# reVault native v1 exporter

Standalone historical exporter used by current reVault releases to read native
vault and archive format v1 and emit encrypted migration-schema-v1 artifacts.

The exporter is pinned to the immutable crates.io releases
`revault_vault_api = 0.0.2` and `revault_lockbox_api = 0.0.2`. The vault API
declares native structure version 1. A bounded, read-only record scanner fills
the public API's enumeration gaps without calling APIs that can lazily mutate
legacy identities. The executable does not import current native formats.
