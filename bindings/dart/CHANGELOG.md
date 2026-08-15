# Changelog

## 0.3.2

- Refresh the checked Rust binding lockfile used by the release validation
  workflow.

## 0.3.1

- Add `Revault.load(nativeLibraryPath: ...)` so Flutter desktop and other
  application bundles can supply the native library location explicitly.
- Continue executable-relative discovery when package-URI resolution is not
  supported by the Dart host.

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
