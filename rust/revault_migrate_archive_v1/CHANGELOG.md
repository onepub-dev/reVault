# Changelog

## Unreleased

- Raised the minimum supported Rust version from 1.88 to 1.95.

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
