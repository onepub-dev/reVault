# reVault API

reVault's core is written in Rust and exposed through native bindings. The APIs use the same model across languages:

* `Revault` provides library-wide initialisation and diagnostics;
* `Vault` manages local Profiles, Contacts and Lockbox records;
* `Profile` represents one of your public/private key identities;
* `Lockbox` manages an encrypted archive; and
* `AgentSession` controls the keys cached by the Session Agent where the binding supports it.

Names follow each language's normal style, so capitalisation and error handling differ slightly. Use the package README for complete examples and the generated API reference for exact signatures.

## Packages and documentation

| Language | Package | Documentation or source |
| --- | --- | --- |
| Dart | [`revault_api`](https://pub.dev/packages/revault_api) | [Dart package documentation](https://pub.dev/documentation/revault_api/latest/) |
| PHP | [`onepub/revault-api`](https://packagist.org/packages/onepub/revault-api) | [revault-php](https://github.com/onepub-dev/revault-php) |
| Go | [`github.com/onepub-dev/revault-api`](https://pkg.go.dev/github.com/onepub-dev/revault-api) | [revault-api](https://github.com/onepub-dev/revault-api) |
| Swift | Swift Package Manager | [revault-swift](https://github.com/onepub-dev/revault-swift) |
| JavaScript | `@onepub-dev/revault-api` | [JavaScript binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/javascript/README.md) |
| TypeScript | `@onepub-dev/revault-api` | [TypeScript binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/typescript/README.md) |
| Python | `revault-api` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/python/README.md) |
| Java | `dev.onepub:revault-api` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/java/README.md) |
| Kotlin | `dev.onepub:revault-api-kotlin` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/kotlin/README.md) |
| C# | `Revault.Api` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/csharp/README.md) |
| C | native SDK | [C binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/c/README.md) |
| C++ | native SDK | [C++ binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/cpp/README.md) |
| Rust | `revault-api` | [crate source](https://github.com/onepub-dev/reVault/tree/master/rust/revault_bindings) |
| Lua | `revault_api` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/lua/README.md) |
| Ruby | `revault_api` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/ruby/README.md) |
| WebAssembly | `@onepub-dev/revault-api-wasm` | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/wasm/README.md) |

The PHP, Go and Swift bindings have their own repositories so their package landing pages, release tags and language-specific documentation can evolve independently. The monorepo retains the shared binding specification and other generated bindings.

## Resource and secret handling

Vault and Lockbox objects own native resources. Close them explicitly when the binding provides `close` or `dispose`, or use the language's scoped resource construct. Do not depend on garbage collection to decide when secret material leaves memory.

API operations within one process do not implicitly use the Session Agent unless the binding exposes and uses `AgentSession`. Closing a Vault or Lockbox object releases that process's resources. Calling `AgentSession.closeLockbox` clears the corresponding key from the Session Agent; these are different operations.

For end-to-end examples of the common operations, see the repository's [API examples](https://github.com/onepub-dev/reVault/blob/master/bindings/API_EXAMPLES.md).
