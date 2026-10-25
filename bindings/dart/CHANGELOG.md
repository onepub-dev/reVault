## 0.6.1

- Fix Ruby frozen gem contents and Fiddle dependency

## 0.6.0

- Record migration dependency publication and private advisory review
- Record packaged format-4 migration qualification
- Prepare security release notes and include format-4 exporter bootstrap
- Update doctor and future-format regressions for format 5
- Complete exchange binding conformance and dependency locks
- Harden Profile backup buffers and cover security boundaries
- Record original #226 history after current-main integration
- Fix #226: integrate full reciprocal exchange across CLI and bindings
- Record completed branch audit and intentional holdbacks
- Merge #322 headless documentation and retained validation evidence
- Merge already-integrated fix branch histories
- Record local 0.6.0 integration validation
- Record local integration in security release checklist
- Merge format-5 descriptions and security work
- Merge plaintext read-only binding support
- Merge October dependency updates
- Expose plaintext read-only archive opening across bindings
- Update Cargo dependencies for October 2026
- Fix #4142: expose unencrypted read-only lockboxes to Java
- Merge #324 mutation regression into main
- Document #324 historical storage fix boundary
- Fix #324: retain mutation regressions and verify deleted files
- Merge #324 historical fix documentation
- Document #324 historical storage fix boundary
- Merge #324 mutation regression into format-5 work
- Fix #324: retain mutation regressions and verify deleted files
- Implement #333: native item descriptions in format 5
- Merge branch 'security/advisory-fixes-2026-10' into issue-333-descriptions
- Document security release and disclosure checklist
- Merge branch 'security/advisory-fixes-2026-10' into issue-333-descriptions
- Fix #333: add multiline descriptions to variables and forms
- Harden key sharing and Vault I/O boundaries
- Fix Swift read-only signing key disposal in conformance
- Fix Dart conformance formatting after Vault merge
- Merge branch 'issue-334-readonly-vault-credentials'
- Merge branch 'fix-clippy-ci-gate'
- Fix Rust Clippy warnings in recovery commands
- Fix #334: test reader locks in Android native conformance
- Fix #334: isolate lock timeouts in process tests
- Fix #334: run reader lock tests on desktop CI
- Fix #334: test read-only Vault locks across processes
- Fix #334: allow credential reads from read-only vaults
- Improve helper guidance and invalid path diagnostics
- Recover verified files and preserve partial ranges with diagnostics
- Add revault-tool install command
- Compile Android acceptance runners before emulator startup
- Use executable DCli SDK for Android release jobs
- Fix Android release setup and candidate test fixtures
- [release candidate] Prepare Dart 0.5.2
- Add Android support for Dart and native C/C++ bindings
- fix: report newer lockbox formats across language bindings
- Repair binding publication retries and preserve tag-triggered releases [skip ci]
- Fix Rust-compatible resolution for terminal crate releases [skip ci]
- Fix #322: bound D-Bus credential and connection waits
- Fix #322: remove native D-Bus dependency from Linux CLI
- Fix #226: add reciprocal invitation-scoped key exchange
- Update standalone Rust binding lockfile for x25519 3
- Merge fa17c691e045693fcc668a25719dff057e58a946 into 689a2bc2a292d02bee819e968768dcbb333ea8a9
- fix(release): normalize Windows package paths
- ci: scope Dart releases to Dart conformance
- fix(release): avoid Windows evidence stack overflow
- fix(windows): avoid Rust TLS during DLL teardown
- style: format Dart conformance with CI SDK
- docs: allow mechanical release metadata changes
- docs: define efficient release monitoring
- release: prepare Dart revault_api 0.3.2
- release: publish Dart revault_api 0.3.1
- ci: add Dart-only release tags
- feat: prepare Dart 0.3 API and standalone vaults
- feat(cli): replace sync with mirror projects
- Add explicit sync rule inspection and clearing
- Clarify sync safeguards and stored rules
- Document target-first sync usage
- Refine sync metadata naming and help
- Fix #267: add safe directory synchronization
- fix native last-error conformance
- fix PHP release conformance
- fix(windows): retry named-pipe reconnect gaps
- fix(lua): decode signed FlatBuffer vtable offsets
- fix(lua): decode signed FlatBuffer vtable offsets
- fix(windows): retry transient agent probes
- fix(kotlin): install Gradle 8.13 on JDK 22
- fix(kotlin): align conformance Gradle version
- fix(windows): clear stale mutex error state
- fix(windows): clear stale mutex error state
- fix(windows): enforce a single session agent
- test(windows): persist native phase output
- fix(dart): match stored profile email in conformance
- test(native): isolate and time conformance phases
- fix(dart): use public package entry point in conformance
- fix(conformance): dispatch Windows agent child
- fix(conformance): dispatch Windows agent child
- test(conformance): bound Windows agent shutdown
- fix(conformance): preserve all moved form values
- fix(release): normalize Windows conformance workspace
- fix(release): support MSVC C conformance build
- fix(release): normalize Windows CMake paths
- fix(release): validate C conformance artifact paths

## 0.5.2

- Add Android native assets for ARM64 devices and x86-64 emulators, requiring
  native API level 23 or later and supporting 16 KB page alignment. Apps must
  also meet their Flutter SDK's minimum API level. ARM32 is excluded.
- Document app-private storage paths and explicit Vault passphrases on Android;
  Android Keystore automatic unlock and the desktop Session Agent are not included.
- Adapt Android secure-memory pools to the app's locked-memory budget while
  retaining memory locking, guard pages, and dump/fork exclusions.
- Report lockboxes written by newer unsupported formats with a clear upgrade
  error while preserving the original archive.

## 0.5.1

- Prepare frozen exporter dependencies before offline CLI tests
- Use registered release identity and resolve Dart helper dependencies
- Repair merged API coverage and fuzz workspace validation
- Publish migration dependencies and retirement notices through trusted CI
- Consolidate historical migrations and validate releases from main
- Merge format-4 development into main and preserve integrated CLI fixes
- Restore deferred format-4 sources for main integration
- Fix published v3 exporter pin and keep legacy fixtures unpublished
- Record published CLI 0.4.1 compatibility line
- Prepare cli release: CLI 0.4.1, bindings
- Regenerate CLI MSI definition for all four binaries
- Keep IPC smoke credentials private and separate build timeout
- Refresh Rust binding locks after native D-Bus removal
- Fix release readiness checks for the format-3 CLI
- Allow release preparation from prerelease source versions
- Merge recovery and alias-list fixes into #327
- Merge main into #327 and preserve standalone creation
- Recover historical archives with current CLI and preserve access
- Merge alias resolution fixes into migration branch
- Fix historical migration unlock and add bulk migration
- Resolve bare lockbox names through local files and aliases
- Update documentation for plural CLI command groups
- Use plural CLI groups and fix Vault container migration
- Restore format-3 compatibility and defer v4 recovery
- Fix #327: register lockbox aliases during create
- Unify lockbox selection and require explicit archive creation
- Fix exact variable moves and renames

## 0.5.0-dev.1

- Reserve the 0.5.x compatibility line for archive format 4 development.
- Retain the release tooling and package fixes from the format-3 releases.

## 0.4.1

- Update the native build-hook dependency to support its current release and
  native-library architecture validation. Archive format remains 3.
- Align the generated package-version metadata with the package manifest.

## 0.4.0

- Bundle the format-3 native engine, compatible with archives produced by
  reVault CLI 0.0.17. Format-4 development changes are excluded.
- Start the 0.4.x archive and Vault compatibility line. Patch releases in this
  line preserve persisted-format compatibility across the CLI and bindings.
- Retain native assets for Linux, macOS, and Windows on x64 and ARM64.

- Hide xtask development tasks behind revault-tool
- Consolidate development commands in revault-tool

## 0.3.15

- Improve release version selection
- Prepare all release: CLI 0.2.0, bindings 0.5.0
- Prepare all release: CLI 0.1.0, bindings 0.4.0
- Match CI test sharding during dependency upgrades
- Serialize dependency upgrade validation tests
- Fix CLI test Clippy warnings
- Remove obsolete DVault design documents
- Add automated dependency upgrades
- Improve release monitoring and fix CLI auto-open test failures
- Prepare cli release: CLI 0.0.19, bindings
- Simplify release commands and report binding publication status
- Refresh fuzz lockfile during release preparation
- Prepare all release: CLI 0.0.18, bindings 0.3.16
- fail matrix validation promptly
- fail release validation faster and show wait progress
- Prepare all release: CLI 0.0.17, bindings 0.3.15
- report release CI failures with job details
- Prepare cli release: CLI 0.0.16, bindings
- Remove the superseded Dart implementation
- Consolidate branch histories and use main as the default
- Add CI release candidates with resumable CLI-first promotion
- Route language binding file operations through Rust
- Require Rust-owned binding operations unless explicitly approved
- Support read-only lockboxes with archive handle locking
- Fix lockbox moves across filesystems and clarify move guidance
- Add password profiles and release CLI 0.0.14 (#306)

## 0.5.0

- Prepare all release: CLI 0.1.0, bindings 0.4.0
- Match CI test sharding during dependency upgrades
- Serialize dependency upgrade validation tests
- Fix CLI test Clippy warnings
- Remove obsolete DVault design documents
- Add automated dependency upgrades
- Improve release monitoring and fix CLI auto-open test failures
- Prepare cli release: CLI 0.0.19, bindings
- Simplify release commands and report binding publication status
- Refresh fuzz lockfile during release preparation
- Prepare all release: CLI 0.0.18, bindings 0.3.16
- fail matrix validation promptly
- fail release validation faster and show wait progress
- Prepare all release: CLI 0.0.17, bindings 0.3.15
- report release CI failures with job details
- Prepare cli release: CLI 0.0.16, bindings
- Remove the superseded Dart implementation
- Consolidate branch histories and use main as the default
- Add CI release candidates with resumable CLI-first promotion
- Route language binding file operations through Rust
- Require Rust-owned binding operations unless explicitly approved
- Support read-only lockboxes with archive handle locking
- Fix lockbox moves across filesystems and clarify move guidance
- Add password profiles and release CLI 0.0.14 (#306)

## 0.4.0

- Match CI test sharding during dependency upgrades
- Serialize dependency upgrade validation tests
- Fix CLI test Clippy warnings
- Remove obsolete DVault design documents
- Add automated dependency upgrades
- Improve release monitoring and fix CLI auto-open test failures
- Prepare cli release: CLI 0.0.19, bindings
- Simplify release commands and report binding publication status
- Refresh fuzz lockfile during release preparation
- Prepare all release: CLI 0.0.18, bindings 0.3.16
- fail matrix validation promptly
- fail release validation faster and show wait progress
- Prepare all release: CLI 0.0.17, bindings 0.3.15
- report release CI failures with job details
- Prepare cli release: CLI 0.0.16, bindings
- Remove the superseded Dart implementation
- Consolidate branch histories and use main as the default
- Add CI release candidates with resumable CLI-first promotion
- Route language binding file operations through Rust
- Require Rust-owned binding operations unless explicitly approved
- Support read-only lockboxes with archive handle locking
- Fix lockbox moves across filesystems and clarify move guidance
- Add password profiles and release CLI 0.0.14 (#306)

# Changelog

## Unreleased

- Preserve explicit and inherited native library overrides across repeated
  `Revault.load()` calls and resolve Dart native extensions through the same
  selected library.
- Add a relocated executable regression test covering native override loading,
  persisted binary content, and translated platform credential errors.

## 0.3.12

- Restore the optional `Revault.load(nativeLibraryPath: ...)` installer hook
  while retaining native-assets loading as the default.
- Allow the bundled native client to use a session agent from a different
  package release when both implement the same agent protocol.
- Added encrypted Lockbox descriptions through `description`, `setDescription`,
  and `clearDescription`. Descriptions use existing encrypted variable storage,
  so the `.lbox` format version is unchanged.
- Add a runnable vault-and-lockbox quick start and clarify installation,
  package examples, and links to the reVault manual.
- Declare WebAssembly as a supported Dart target.

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
