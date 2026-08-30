# Changelog

## 0.0.8 - 2026-08-30

- Pin the Linux Secret Service implementation to the Rust-1.88-compatible
  5.1.0 release so a fresh dependency resolution cannot select `aes 0.9.3`,
  which requires Rust 1.89.

## 0.0.7 - 2026-08-30

- Persist the vault container signer separately from the removable `default`
  profile so deleting or restoring that profile cannot strand the vault.
  Existing current-format vaults add the dedicated record only after an
  authenticated open, while missing or mismatched signer material fails closed.
- Reject vault password changes when the vault structure version is not the
  current version, before changing any key material.
- Allow migration import to create a vault container with its restored owner
  signing key instead of an unrelated ephemeral signer.
- Treat the session-agent protocol version as the compatibility contract so
  clients and agents from different package releases can interoperate when
  their wire protocol is unchanged.

## 0.0.4

- Restored the minimum supported Rust version to Rust 1.88.
- Renamed the cryptographic identity model to profile throughout the public API
  and encrypted vault format, including profile generations, history, email
  records, publishing fields, backup labels, and owner-signing cache scope.
  The vault structure version is now 2 and vault v1 is the first supported
  migration source.
- Added stable vault-version probing and actionable unsupported-version errors,
  allowing migration tooling to identify an old vault without asking the
  current API to interpret its version-specific records.
- Added a read-only encrypted vault view for metadata consumers such as command
  completion. It lists profile names, contacts, reusable forms, and known
  lockboxes without attaching or loading owner-signing material.
- Extended the Lockbox Session Agent with typed, vault-scoped cache entries for
  vault unlock secrets and owner-signing keys.
- Reused the existing zeroizing secret storage, TTL renewal, expiry, suspend
  cleanup, stop/forget behavior, and same-user IPC boundary. Typed secret
  entries are excluded from normal open-lockbox listings.
- Added cache invalidation for invalid vault secrets, password replacement,
  vault replacement, profile removal, and signing-key rotation.
- Disabled auto-open now prevents typed vault and signing caches from starting
  or using the session agent. Existing archive content-key cache behavior and
  CI/agentless operation remain unchanged.
- Added an explicit agent start operation so the agent can be launched before a
  vault is opened, avoiding inherited vault file locks.
- Added an agent compatibility handshake. Clients automatically stop and
  replace an incompatible agent left running by a previous CLI installation;
  replacement clears the old in-memory secret cache.
- Added tests for typed-entry TTL and listing isolation, disabled-agent policy,
  transport validation, cleanup, and Windows compatibility.
