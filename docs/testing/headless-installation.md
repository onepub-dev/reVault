# Headless installation regression (#322)

Scope: source branch `issue-322-headless-cli`, based on `0f9a137e`.
This is verification evidence, not a statement that published packages include
this fix. No archive or Vault encoding changes are made.

## Cause and change

The Linux sleep watcher used `dbus 0.9.12`, which pulled `libdbus-sys` into the
normal CLI build. Its build script required `dbus-1.pc`. Credential storage was
already using the Rust `zbus` backend. The watcher now also uses `zbus`, retaining
logind signal filtering and file-descriptor inhibitors. The delay inhibitor stays
held until the suspend handler has cleared cached secrets; resume reacquires it.
Method calls use a five-second timeout. Missing logind remains an unavailable
optional capability, as before.

Do not remove all `libdbus-sys` entries from Cargo.lock: historical migration
readers still have their own dependencies. Inspect the normal CLI dependency
graph, separately from workspace development and historical exporter builds:

```sh
cd rust
cargo tree --locked -p revault_cli --edges normal,build -i libdbus-sys
cargo test --locked -p revault_vault_api@0.0.42 --lib
cargo test --locked -p revault_cli --test headless -- --nocapture
```

The sleep-watcher integration test requires the `dbus-daemon` executable as a
**test-only** fixture. It starts its own private bus and synthetic logind service;
it neither accesses the host's logind nor requests real system sleep. The test
checks signal delivery, delay-inhibitor lifetime, resume reacquisition and block
inhibitor lifetime. Unit tests of secret storage use an in-memory store and do
not establish integration with a desktop keychain.

## Platform audit

| Platform/carrier | Build dependency and runtime boundary | Evidence / outstanding work |
| --- | --- | --- |
| Linux CLI/native Vault | Pure-Rust zbus/Secret Service clients; logind and credential service are optional at runtime | Clean container installation, linkage inspection, missing-bus CLI lifecycle and private-logind test pass locally |
| macOS CLI/native Vault | Apple frameworks via `objc2`/keyring; no Linux dbus dependency | Source audit only here. Existing macOS platform CI must run; SSH/locked Keychain behavior still needs platform validation |
| Windows CLI/native Vault | Windows system APIs via windows-sys/keyring; no Linux dbus dependency | Source audit only here. Existing Windows platform/credential CI must run; service-account and denied-store behavior still needs platform validation |
| Native bindings | Share the Vault crate when built with Vault integration | Dependency removal applies to that crate; packaged carrier interoperability remains a release check |
| WASM core | Linux/macOS/Windows target-specific dependencies excluded | No desktop store or suspend guarantee is added to browser APIs |

The Linux test removes display variables, supplies nonexistent private session
and system bus addresses, closes stdin, and supplies only synthetic credentials.
It checks persisted bytes through separate CLI processes after stopping the agent,
including replacement, unchanged replacement, deletion, explicit Auto Open refusal,
missing credentials and successful recovery with explicit credentials. Each command
must return within 30 seconds (the shared process harness has a 180-second kill
limit). `REVAULT_HEADLESS_CLI` selects an installed binary for the same test.

## Clean build evidence and CI

Local validation used `rust:1.88-bookworm` (image digest
`sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`)
with a fresh target directory and this source tree mounted read-only. Registry
source downloads were cached; compiled host artifacts were not reused.
`dpkg-query` confirmed `libdbus-1-dev` absent. Ordinary default-feature
`cargo install --locked --path revault_cli --debug` succeeded. `ldd lockbox`
listed libc, libm, libgcc_s and the loader, with no libdbus dependency.

The `headless-install` job in `.github/workflows/rust.yml` repeats the absence
check, uses the ordinary release-mode install, inspects linkage, then runs the
CLI lifecycle against that installed binary. Existing workspace jobs retain
native dependencies needed by historical readers. The new job must continue
without installing D-Bus development packages.

The installed-binary test is registered in `revault_tooling` behind
`installed-cli-tests` so it can run without compiling the CLI's historical
migration dev-dependencies. The same test also runs normally in `revault_cli`.
The installed debug executable passed this test inside the clean container
(78.23 seconds total, every command below the 30-second bound). All 25 Vault
unit tests and 23 Vault integration tests passed locally, and affected-package
Clippy passed with warnings denied. Release-mode CI and other operating systems
have not been run locally.

This fixes the reported Linux build dependency. It does not establish bounded
behavior for every locked/denied/hung desktop credential service: those broader
H1 acceptance cases remain outstanding. In particular, the pre-existing Secret
Service unlock-prompt paths were not redesigned by this change. Explicit Vault
credentials remain the tested noninteractive alternative; there is no plaintext
credential fallback.
