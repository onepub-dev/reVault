# Changelog

## 0.0.4 - 2026-08-30

- Restored the minimum supported Rust version to Rust 1.88.
- Added archive migration schema 2 with explicit directories, directory and
  symlink permissions, and optional archive descriptions while retaining
  schema-1 decoding compatibility.

## 0.0.1

- Added the native-API-independent encrypted streaming migration envelope and
  logical vault/archive migration-schema-v1 records.
- Added authenticated completion trailers, bounded frames, corruption checks,
  and zeroizing secret record fields.
