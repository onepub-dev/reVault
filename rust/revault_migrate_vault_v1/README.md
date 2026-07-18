# reVault native v1 exporter

Standalone historical exporter used by current reVault releases to read native
vault and archive format v1 and emit encrypted migration-schema-v1 artifacts.

The exporter is pinned to the immutable crates.io releases
`revault_vault_api = 0.0.2` and `revault_lockbox_api = 0.0.2`. The vault API
declares native structure version 1. A bounded, read-only record scanner fills
the public API's enumeration gaps without calling APIs that can lazily mutate
legacy identities. The executable does not import current native formats.
