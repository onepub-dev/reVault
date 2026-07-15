# reVault language bindings

The native bindings are intentionally based on one C ABI. Build the internal
`revault_bindings` Rust crate from `rust/` and use
`rust/revault_bindings/revault_api.h` with
Dart FFI, Java 22+ Foreign Function & Memory, or Python `ctypes`.

Symbols use descriptive namespaces: `lockbox_*`, `vault_*`, `key_*`, and
`buffer_*`. The Rust crate names remain `revault_lockbox_api` and
`revault_vault_api`.

The ABI covers lockbox data and metadata operations, key management, recovery,
vault directories, local-vault integration, session-agent controls, platform
secret-store controls, key formatting, and explicit buffer/handle ownership.
Structured responses use the shared `LBWF` binary frame and Protobuf schemas in
`bindings/proto`. Every structured result and form-definition input uses a
named Protobuf message. File contents and cryptographic material remain raw
bytes.
The hosted WebAssembly package runs every API call through a real WASM
dispatcher and uses its Node host adapter for filesystem, native vault,
keyring, and agent facilities. The standalone browser module retains portable
lockbox/key operations, but browser-only execution is not represented as the
complete API because browsers cannot provide vault directories, an OS keyring,
or a session-agent process.

## Language targets

The native ABI and shared Protobuf schema are the common contract for the
selected issue-189 targets:

| Target | Binding location |
| --- | --- |
| JavaScript / TypeScript | `bindings/javascript` (one public package); `bindings/typescript` is its private compile-time conformance consumer |
| Python | `bindings/python` |
| Java / Kotlin | `bindings/java`, `bindings/kotlin` |
| C# | `bindings/csharp` |
| C++ / C | `bindings/cpp`, `bindings/c` |
| PHP | `bindings/php` |
| Go | `bindings/go` |
| Rust | `bindings/rust` |
| Lua | `bindings/lua` |
| Ruby | `bindings/ruby` |
| Dart | `bindings/dart` |
| Swift | `bindings/swift` |
| WebAssembly | `bindings/wasm`, `rust/revault_wasm_bindings` |

The checked-in binding surfaces and Protobuf models are validated or regenerated
through the Rust tool:

```text
cargo run --locked -p revault_tooling --bin revault-tool -- bindings check
cargo run --locked -p revault_tooling --bin revault-tool -- bindings generate-protobuf
```

Full model regeneration requires `protoc` plus the pinned Go, Dart, and Swift
Protobuf generator plugins on `PATH`; release containers install and run those
same generators before compiling their package consumers.

The generated declaration surface covers all 211 exported C ABI functions:
the ABI-version query plus 210 domain functions. C++, C, PHP, Swift, Lua, and Ruby
consume the header or the complete native symbol table directly; Java/Kotlin
use the generated Java FFM method-handle surface and typed facade.
The generated low-level surfaces are intentionally separate from the typed
facades so new ABI additions cannot silently disappear from a target.

The acceptance contract for language-level archive/vault interoperability is
defined in [`e2e/CONFORMANCE.md`](e2e/CONFORMANCE.md). Language runners must
execute that contract; symbol coverage and syntax checks are not substitutes
for e2e coverage.

## Distribution and publication

All foreign-language packages use the same versioned C ABI library. Canonical
native archives contain the dynamic and static libraries, the Windows DLL
import library where applicable, the target-built Ruby ABI adapter, C header,
Protobuf schema, license, target
metadata, SPDX SBOM, and SHA-256 sidecar. Linux requires
the system `libdbus-1` runtime; macOS and Windows use their native secret-store
implementations. A package must never select an artifact for another operating
system, architecture, C runtime, or ABI. `REVAULT_LIBRARY` is a development-only
override and is deliberately unset during package acceptance tests.

| API | Public endpoint | Native delivery |
| --- | --- | --- |
| JavaScript and TypeScript | npm `@onepub/revault-api` | optional `@onepub/revault-api-native-<target>` carrier |
| WebAssembly | npm `@onepub/revault-api-wasm` | bundled WASM and installed native host carrier |
| Python | PyPI `revault-api` | one wheel per platform tag |
| Java | Maven Central `dev.onepub:revault-api` | native resources extracted by the loader |
| Kotlin | Maven Central `dev.onepub:revault-api-kotlin` | Java runtime artifact dependency |
| C# | NuGet `OnePub.Revault.Api` | `runtimes/<rid>/native` assets |
| Dart | pub.dev `revault_api` | checked native assets selected by `Vault.load()` |
| Ruby | RubyGems `revault_api` | platform-specific gems |
| PHP | Packagist `onepub/revault-api` | Composer package containing checked native assets |
| Lua | LuaRocks `revault_api` | platform rocks containing checked native assets |
| Go | `github.com/onepub-dev/revault-api-go` | statically linked packaged native libraries |
| Rust | crates.io `revault-api` | native Rust source; no C ABI dependency |
| Swift | Swift Package Manager product `RevaultAPI` | macOS XCFramework or Linux native package |
| C | GitHub Releases, vcpkg, Homebrew, Debian and RPM | canonical SDK archive and `revault_api.h` |
| C++ | ConanCenter, vcpkg, Homebrew, Debian and RPM | class facade over the canonical C ABI |

The six canonical targets are Linux glibc x86-64/ARM64, macOS x86-64/ARM64,
and Windows MSVC x86-64/ARM64. Registry credentials are release authority and
must use trusted publishing/OIDC where available, otherwise environment-scoped
least-privilege secrets. Credentials are never committed.

### Rust release tooling

All repository-owned generation, validation, packaging, installation, and
publication automation is exposed by the unpublished Rust workspace binary:

```text
cargo run --locked -p revault_tooling --bin revault-tool -- --help
```

Package-manager recipes such as `conanfile.py`, formula templates, gemspecs,
rockspecs, and CMake files remain in their ecosystem-required formats; they are
inputs consumed by the Rust tool rather than release scripts.

### Release acceptance and order

1. Build release-mode native libraries for all six targets and create immutable,
   deterministic native archives with `revault-tool release package-native`.
2. Verify and securely unpack those archives, then stage and assemble every
   ecosystem package exclusively from them.
3. Install every claimed language/target package in a clean GitHub Actions
   consumer with no build tree or native-library override. The test must record
   the installed native path and archive hash before exercising all 210 calls.
   Rust is the explicit source-native exception: its consumer runs the complete
   `public_api_suite` and `vault_api` suites after `cargo package`, securely
   unpacks the `.crate` into a clean consumer, and records that archive's hash;
   it never substitutes the C conformance executable or claims a native-library
   installation.
4. Run the canonical Linux x86-64 all-pairs matrix: each of sixteen consumers
   opens the archive and vault from every other producer (480 directed paths).
5. Attest the accepted artifacts, publish native carriers first, language
   facades second, and hosted WASM last. Registry publication is never used as
   a substitute for pre-publication package installation tests.

The full claimed package matrix contains 94 combinations: every language on
all six native targets except Swift's two unclaimed Windows targets. A missing
runtime or preview runner blocks publication; it is not converted to a skip or
ABI-only smoke test. The detailed operation contract is in
[`e2e/CONFORMANCE.md`](e2e/CONFORMANCE.md).

Registry setup references: [npm](https://docs.npmjs.com/creating-and-publishing-scoped-public-packages/),
[Python wheels](https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/),
[Maven Central](https://central.sonatype.org/publish/publish-portal-guide/),
[Dart](https://dart.dev/tools/pub/publishing),
[RubyGems](https://guides.rubygems.org/publishing/),
[Go modules](https://go.dev/doc/modules/publishing),
[crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html), and
[ConanCenter](https://docs.conan.io/2/devops/using_conancenter.html).
