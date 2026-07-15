# Changelog

## Unreleased

- Raised the minimum supported Rust version from 1.88 to 1.95.
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
