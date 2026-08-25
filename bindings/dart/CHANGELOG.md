# Changelog

## Unreleased

- Restore the optional `Revault.load(nativeLibraryPath: ...)` installer hook
  while retaining native-assets loading as the default.
- Allow the bundled native client to use a session agent from a different
  package release when both implement the same agent protocol.
- Added encrypted Lockbox descriptions through `description`, `setDescription`,
  and `clearDescription`. Descriptions use existing encrypted variable storage,
  so the `.lbox` format version is unchanged.

## 0.3.6

- Add process-local lockbox handles with deterministic `close` support.
- Add agent-backed lockbox handle acquisition without extending the agent TTL.
- Add explicit platform credential context support for sudo-safe vault access.

## 0.3.5

- Throw `VaultPassphraseAccessException` with platform credential-store recovery
  guidance when a remembered Vault passphrase cannot be retrieved.

# 0.3.3
- feat(dart): bundle native rust libraries with build hooks
- docs(bindings): added worked examples to each method.
- fix(release): normalize Windows package paths
- fix(release): avoid Windows evidence stack overflow
- Fix #270: define identifier case sensitivity
- Fix #271: disallow variable directory collisions
- Fix #269: accept move destination directories (#274)


## 0.3.2
- Refresh the checked Rust binding lockfile used by the release validation
  workflow.
- Use Dart native assets and build hooks to bundle the target-specific carrier.

## 0.3.1

## 0.3.0

- Breaking: rename the runtime entry point from `Vault` to `Revault` and use
  `Vault` exclusively for the persistent local security store.
- Breaking: create and open archives through `Lockbox.create`, `Lockbox.open`,
  `Lockbox.createInMemory`, and `Lockbox.openBytes`.
- Breaking: accept `SecretString` for passphrases and passwords and
  `SecretBytes` for binary keys so callers can wipe owned secret buffers with
  `close()`.
- Add `SecretBytes.fromString` for UTF-8 text stored in binary secret fields.
- Breaking: name Vault signing identities `ProfileSigningKeyPair` and
  `ProfileSigningPublicKey`; retain “owner” only for the Lockbox owner role.
- Replace the misleading `LocalVault` facade with the explicit singleton
  `AgentSession`. Ordinary lockbox operations no longer use the agent.
- Add process-local `Lockbox.close` and `Vault.close`; agent-held keys are
  managed independently with `AgentSession.closeLockbox` and `closeAll`.
- Allow credential-free `Lockbox.open(path)` to open the default Vault through
  its platform-stored passphrase and resolve the lockbox credential there; it
  never consults the agent.
- Replace stringly typed key formats, cache policies, worker policies, workload
  profiles, and agent activity kinds with Dart enums.
- Throw `RevaultException` with structured native details instead of requiring
  callers to inspect `lastError` and `lastErrorDetails`.
- Expand public API documentation, including credential provenance, secret
  lifetimes, and the security consequences of platform-stored passphrases.
- Keep generated native operations under `lib/src` so they are not package API.
- Expose only `package:revault_api/revault_api.dart` as a public Dart library.

See `UPGRADING.md` for the complete 0.2.x migration guide.

## 0.2.0

- Initial complete lockbox and vault API binding.
- Initial binary structured transport and concrete result models.
- Linux, macOS, and Windows native runtime discovery for x86-64 and ARM64.
