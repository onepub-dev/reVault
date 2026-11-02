# Native dependency audit — November 2, 2026

Issue: [#215](https://github.com/onepub-dev/reVault/issues/215).
Reviewed main `88a55a428bc35e040a4be7840ae043c5897ce823` with locked Cargo
metadata and dependency trees. The strict objective that every dependency is
implemented entirely in Rust is **not currently met**.

| Dependency | Scope | Finding |
| --- | --- | --- |
| `ring 0.17.14` | Active networking | Bundled C and assembly, built with `cc`; used by rustls/ureq for clients and by the key server's TLS/ACME stack. |
| `libdbus-sys 0.2.7` | Historical v1/v2 migration executables on Linux | Links the system C D-Bus library through `dbus`; its vendored feature can also compile C. These frozen exporter graphs must not be silently rewritten. |
| `zstd-sys 2.1.0+zstd.1.5.7` | Optional historical native encoder | Reached through `zstd` when `native-zstd-encoder` is enabled for v1/v2 migration packages. Not in the default workspace normal/build graph. |
| `alloca 0.4.0` | Development/benchmarks | Compiles `alloca.c`; reached through Criterion. Not a normal/build dependency of the shipped archive core. |
| `iana-time-zone-haiku 0.1.2` | Haiku-specific dependency | Compiles C++; visible with `--target all`, not a supported release target. |
| OS FFI crates | Platform integration | `libc`, Windows API and Apple framework bindings call native OS libraries. Rust wrappers do not make the OS implementation pure Rust. |

The current `revault_lockbox_api` normal/build dependency graph has no `cc`
ancestor, including when queried with `--target all`. Its active Zstd codec is
`zstd-complete 0.3.0`, not the optional historical native encoder. This narrow
observation must not be generalized to the networking layer, benchmark graph,
or a proof that all Rust dependencies are free of unsafe code.

Resolved-package source inspection also found C examples/test fixtures in
`nix`, `sysinfo` and `walkdir`; their presence does not mean they are compiled
into reVault. `cc` contains a compiler-identification C source. Build scripts
alone likewise do not establish a native-code dependency: many only generate
Rust configuration or data.

## Follow-up decisions

1. Keep #215 open: the strict pure-Rust claim cannot be verified as true.
2. Separate a policy for shipped application/cryptographic code from OS FFI,
   historical migration compatibility and development-only tools. Do not
   silently declare those categories exceptions to the original objective.
3. If strict pure-Rust networking is required, qualify a replacement TLS crypto
   provider across ureq, rustls, ACME, SMTP, native bindings and supported
   platforms. This requires compatibility/security testing; it is not a safe
   incidental dependency substitution during the current release.
4. Preserve frozen migration graphs and evidence. Any exporter change needs
   its own version and packaged migration qualification.
5. Once the intended policy is agreed, enforce it on resolved release graphs
   in CI, with explicit reviewed exceptions. Checking Cargo.lock names or
   searching only for `cc` is insufficient to detect system-library linking.

## Evidence and limits

The adjacent [dependency-tree evidence](../evidence/native-dependency-audit-2026-11-02/trees.txt)
records default normal/build trees, all-target inspection, development-only
paths and optional encoder paths. Commands use `--locked --offline`; no
dependency versions or production code were changed. Source inventories used
the exact locally cached registry packages resolved by Cargo metadata.

This is a source/dependency-graph audit, not new cross-platform builds, an audit
of every unsafe Rust block, or a claim that OS libraries contain no native code.
Language facade source and external release tools such as FlatBuffers' compiler
are outside the narrower Rust dependency objective and must not be described as
pure Rust either.
