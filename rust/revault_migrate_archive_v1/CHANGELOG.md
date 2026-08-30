# Changelog

## 0.0.4 - 2026-08-30

- Restored the minimum supported Rust version to Rust 1.88.
- Emit archive migration schema 2 so empty directories and custom directory or
  symlink permissions survive migration.
- Use IPC protocol 3 to accept contact key records from the current CLI, so
  contact-only v1 archives remain migratable after the vault is upgraded.

## 0.0.2

- Open old archives using every profile key generation in the migrated vault,
  with the session-agent content key as an optional fast path and
  `LOCKBOX_PASSWORD` reserved for password-only archives.
- Updated the secret IPC capability to protocol 2 so the vault credential is
  received through framed stdin rather than arguments or environment values.

## 0.0.1

- Added a streaming archive-format-v1 exporter that preserves archive content
  keys and access directories while excluding old commit/signature history.
- Added versioned capabilities and framed child-process secret input.
