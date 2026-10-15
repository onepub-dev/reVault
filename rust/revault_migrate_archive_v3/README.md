# Deprecated historical migration package

Use [revault_migrate_v3](https://crates.io/crates/revault_migrate_v3) through `lockbox doctor migrate` instead. Archive format 2 is handled by `revault_migrate_v2`; archive format 3 and Vault structure 3/container 3 are handled by `revault_migrate_v3`.

Historical readers are now bundled privately inside one exporter package per archive/container version. This package is retained for reproducible historical dependencies and regression fixtures; it is excluded from normal releases. Existing published versions remain available and are not yanked. Archive/container version and Vault structure version are separate: container 1 can hold Vault structures 1 or 2.
