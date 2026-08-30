# Changelog

## 0.0.3 - 2026-08-30

- Restored the minimum supported Rust version to Rust 1.88.
- Accept migration format 0.0.4 so the v1 exporter remains installable with
  the current migration coordinator.

## 0.0.1

- Added a read-only vault-format-v1 exporter pinned to the immutable
  `revault_vault_api 0.0.2` and `revault_lockbox_api 0.0.2` releases.
- Added migration of pre-history identities, all form revisions, orphan local
  records, and a versioned child-process capability/secret protocol.
