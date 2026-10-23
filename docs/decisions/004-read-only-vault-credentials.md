# 004: Read-only Vault credential access

Status: accepted for implementation, 2026-10-22. Issue: [#334](https://github.com/onepub-dev/reVault/issues/334).

## Context

`ReadOnlyVault` originally exposed names and other discovery metadata. The
implementation comment called it a metadata-only view, but this restriction is
an API policy rather than an archive or cryptographic constraint. Opening a
Vault with the read-only Lockbox state already permits decrypting secret
variables. A caller that only needs stored credentials currently has to open a
writable Vault, acquire its exclusive lock, and may block other processes while
waiting for user input.

## Decision

The existing read-only mode may read stored credentials. Read-only means the
handle cannot create, replace, delete, or persist Vault records or populate the
Session Agent cache. It does not
mean that decrypted secrets are unavailable to a caller that supplied the Vault
passphrase. No additional open mode is introduced.

Read-only handles keep the archive's shared file lock for their lifetime. They
can coexist with other read-only handles; a writer waits for them to close.
Callers should close a Vault handle before an interactive wait when they no
longer need it. Returning a credential does not require keeping its Vault
handle open: the caller owns the returned secret or key and must dispose of it
according to the language binding's normal lifetime rules.

The writable profile signing-key loader retains its legacy behavior: it may
create and persist a missing signing key. The read-only loader returns only a
key already stored in the Vault. If none exists, it returns the existing
`NotFound` error, mapped to the structured `not_found` category at the C ABI.
It never generates a temporary key, writes a key, or populates
the Session Agent cache. A temporary generated key would be a broken signing
identity because it could not be recovered after the call.

The same read-only contract applies to current and historical private keys,
current and historical profile signing keys, password profiles, and remembered
Lockbox passwords. Historical reads use existing records only; they must not
trigger the writable migration path that creates profile history. Missing
records use existing errors and optional-return conventions. No new Rust error type
or archive/Vault format change is required.

## Consequences

Possession of the Vault passphrase and a read-only handle grants access to
stored secret material. API documentation must state this plainly. Code that
needs metadata alone can continue using the same handle and only call listing
methods. Applications requiring credentials can use a shared reader while
another reader remains active, but they cannot bypass an active writer or keep
a writer from waiting if they retain the handle indefinitely.

The Rust API, C ABI, and public language bindings must expose the same reads
with their normal secret ownership and error mappings. Tests must verify
credential parity, concurrent readers, writer exclusion, and that a missing
signing key remains absent after a read-only attempt.
