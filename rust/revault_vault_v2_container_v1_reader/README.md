# Deprecated historical migration package

Use [revault_migrate_v1](https://crates.io/crates/revault_migrate_v1) through `lockbox doctor migrate` instead.

Historical readers are now bundled privately inside one exporter package per archive/container version. This package is retained for reproducible historical dependencies and regression fixtures; it is excluded from normal releases. Existing published versions remain available and are not yanked. Archive/container version and Vault structure version are separate: container 1 can hold Vault structures 1 or 2.

---

# Historical migration compatibility reader

This package preserves a specific published historical implementation for migration; it is not the current reVault application API. See `PROVENANCE.json` for the exact original package and registry checksum. The archive readers use patched ruzstd 0.9.1 with an explicit 100 MiB decoder window admission limit (also applied to the first frame), preserving bounded output and concatenated-frame decoding. This window limit is not a total-process RSS guarantee. Original license and source are retained. Historical external examples, integration tests and benchmarks are not packaged; focused decoder tests are explicitly invoked, while retained public migration tests qualify supported formats. No historical fixture is regenerated.

The original upstream README follows.

# revault_vault_api

Public Rust API for creating and managing reVault local vaults.

The vault API stores profiles, contacts, signing keys, lockbox key-directory
backups, and content-key integrations. It also provides platform secret-store
integration and the local open-cache/session service.

Use it above `revault_lockbox_api` when an application needs local profile and
key lifecycle management, rather than only direct access to a `.lbox` archive.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

See the repository license for licensing terms.
