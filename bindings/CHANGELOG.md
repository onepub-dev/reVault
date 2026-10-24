# Binding changelog

## Unreleased

- Move CLI and bindings to compatibility line 0.6.x with archive format 5.
  Migrate historical archives and Vaults using the current CLI before opening
  them through the bindings. Retain migration backups.
- Provide reciprocal exchange through every supported language facade, including
  invitation, acceptance, independent fingerprint verification, listing,
  cancellation, forgetting and archive-author verification. Credential sessions
  do not retain a Vault file lock while waiting for network or user interaction.
- Include hardened Vault sidecar locking, private backup output and bounded
  backup restoration in the native carriers. Replace installed native libraries
  as well as facade packages when upgrading.

- Add Android ARM64 and x86-64 native SDK artifacts for Dart/Flutter and C/C++,
  with API level 23 minimum and 16 KB page alignment. Other language packages
  retain their desktop support; Android Java/Kotlin requires a separate bridge.
- Keep the native library selected by the runtime loader across facade
  factories and explicit reloads, including relocated installations and
  inherited library overrides.
- Bind the Ruby shim to the selected native carrier instead of linking a
  second copy, and exercise persisted binary content in every loader smoke
  test, including WebAssembly.
- Update native dependencies for vault structure v3. Existing key-pair profile
  operations retain their meaning and cannot overwrite password profiles.
- Password profile management is currently available through the CLI and Rust
  vault API. Foreign-language profile methods continue to manage key pairs;
  existing password-based lockbox operations remain available.

## 0.3.13 - 2026-08-30

- Correct native FFI safety contracts for caller-owned pointers and returned
  buffers, and update generated language documentation with operation-specific
  descriptions and ownership guidance.
- Align binding examples and facades with the current signed Lockbox and Vault
  APIs without adding legacy-format handling to the core bindings.
- Establish caller-supplied owner signing keys in the first commit of password-
  and contact-protected Lockboxes, preserving owner continuity on later writes.
- Preserve the advertised Rust 1.88 minimum in fresh Rust package resolutions
  by using `revault_vault_api 0.0.8`.

## 0.3.12

- Fix the C# package's private FlatBuffers namespace so the public
  `Revault.Revault` class no longer prevents the binding from compiling.
- Add a runnable Dart vault-and-lockbox quick start and clarify installation,
  platform support, package examples, and manual links across the language
  binding READMEs.
- Declare Dart WebAssembly support and correct Dart and Go source formatting
  required by their registry validation gates.
- Publish the Composer companion repository through Packagist's update API and
  retain automatic updates through its GitHub integration.
- Harden the binding release promotion workflow and parallelize the Linux CLI
  build and test phases.

## 0.3.0

- Document every supported facade member across C, C++, C#, Dart, Go, Java,
  Kotlin, JavaScript/TypeScript, Lua, PHP, Python, Ruby, Rust, Swift, and WASM.
- Make low-level native operation layers internal where each language permits.
- Enforce public API and canonical schema documentation in the binding gate.
- Treat the resulting source-level visibility changes as a pre-1.0 minor
  release for every binding package.

## 0.2.0

- Initial complete lockbox and vault binding release.
