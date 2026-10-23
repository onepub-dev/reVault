# reVault API

reVault's core is written in Rust and exposed through native bindings. The APIs use the same model across languages:

* `Revault` provides library-wide initialisation and diagnostics;
* `Vault` manages local Profiles, Contacts and Lockbox records;
* `Profile` represents one of your public/private key identities;
* `Lockbox` manages an encrypted archive; and
* `AgentSession` controls the keys cached by the Session Agent where the binding supports it.

Names follow each language's normal style, so capitalisation and error handling differ slightly. Use the package README for complete examples and the generated API reference for exact signatures.

## Packages and documentation

| Language    | Package                                                                                     | Documentation or source                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Dart        | [`revault_api`](https://pub.dev/packages/revault_api)                                       | [Dart package documentation](https://pub.dev/documentation/revault_api/latest/)                              |
| PHP         | [`onepub/revault-api`](https://packagist.org/packages/onepub/revault-api)                   | [revault-php](https://github.com/onepub-dev/revault-php)                                                     |
| Go          | [`github.com/onepub-dev/revault-api`](https://pkg.go.dev/github.com/onepub-dev/revault-api) | [revault-api](https://github.com/onepub-dev/revault-api)                                                     |
| Swift       | Swift Package Manager                                                                       | [revault-swift](https://github.com/onepub-dev/revault-swift)                                                 |
| JavaScript  | `@onepub-dev/revault-api`                                                                   | [JavaScript binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/javascript/README.md) |
| TypeScript  | `@onepub-dev/revault-api`                                                                   | [TypeScript binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/typescript/README.md) |
| Python      | `revault-api`                                                                               | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/python/README.md)                |
| Java        | `dev.onepub:revault-api`                                                                    | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/java/README.md)                  |
| Kotlin      | `dev.onepub:revault-api-kotlin`                                                             | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/kotlin/README.md)                |
| C#          | `Revault.Api`                                                                               | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/csharp/README.md)                |
| C           | native SDK                                                                                  | [C binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/c/README.md)                   |
| C++         | native SDK                                                                                  | [C++ binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/cpp/README.md)               |
| Rust        | `revault-api`                                                                               | [crate source](https://github.com/onepub-dev/reVault/tree/master/rust/revault_bindings/README.md)            |
| Lua         | `revault_api`                                                                               | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/lua/README.md)                   |
| Ruby        | `revault_api`                                                                               | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/ruby/README.md)                  |
| WebAssembly | `@onepub-dev/revault-api-wasm`                                                              | [binding README](https://github.com/onepub-dev/reVault/blob/master/bindings/wasm/README.md)                  |

The PHP, Go and Swift bindings have their own repositories so their package landing pages, release tags and language-specific documentation can evolve independently. The monorepo retains the shared binding specification and other generated bindings.

## Resource and secret handling

Vault and Lockbox objects own native resources. Close them explicitly when the binding provides `close` or `dispose`, or use the language's scoped resource construct. Do not depend on garbage collection to decide when secret material leaves memory.

API operations within one process do not implicitly use the Session Agent unless the binding exposes and uses `AgentSession`. Closing a Vault or Lockbox object releases that process's resources. Calling `AgentSession.closeLockbox` clears the corresponding key from the Session Agent; these are different operations.

For end-to-end examples of the common operations, see the repository's [API examples](https://github.com/onepub-dev/reVault/blob/master/bindings/API_EXAMPLES.md).

## Item descriptions

Lockboxes expose descriptions for variables, form records, form definitions,
and field definitions. Vaults expose reusable definition and field descriptions.
Fields have definition guidance only; individual field values do not have their
own description.

The language facades expose `itemDescription` (or the language's
`item_description`/`ItemDescription` spelling). For example, in Dart:

```dart
box.itemDescription('variable', 'DB_HOST',
    operation: 'set', text: 'Database host\nUse the internal hostname.');
box.itemDescription('field', 'login',
    operation: 'set', field: 'username', text: 'Full email address');
box.commit();
final guidance = box.itemDescription('field', 'login', field: 'username');
box.itemDescription('record', '/work/github', operation: 'clear');
box.commit();
```

Targets are `variable`, `record`, `definition`, and `field`. Operations are
`get`, `set`, and `clear`. Supply `text` only for `set` and `field` only for a
field target. The result is the current optional text. Clearing a definition
returns an empty string, following its existing description model; clearing
other targets returns no value. Missing owners are errors. Lockbox writes must
be committed; Vault writes commit immediately. Read-only Lockboxes can read
descriptions and reject writes using the existing invalid-operation error.

Native callers use `lockbox_item_description` or
`vault_directory_item_description` with a UTF-8 JSON request containing those
five keys (`reference` is the variable name, record path, or definition
reference). Optional keys may be omitted or null. Unknown keys and invalid
combinations are rejected. Results use the existing `OptionalString` FlatBuffer;
free them with `buffer_free`. Errors use the existing error channel and a null
result buffer. JSON requests are limited to eight mebibytes; decoded descriptions
remain subject to the one-mebibyte normal-variable limit. Field descriptions for
one revision share that limit as an encoded map.

No archive format or FlatBuffers schema change is required. Older format-4
clients retain the encrypted metadata during ordinary updates, but do not
maintain descriptions when renaming/deleting their owners or revising definitions.

## Transaction maintenance and v4

The v4 core changes storage behavior without adding C ABI operations or changing the FlatBuffers schema. All language packages need a native carrier rebuilt with that core; generated facade signatures do not need regeneration solely for this change. Migrate the Vault first and then older Lockboxes with the CLI before opening them using a v4 library.

| Rust core API change | Shared binding status |
| --- | --- |
| New `Lockbox::compact()` | Rust/CLI only. Rebuilds and verifies live state, commits pending changes, replaces storage and discards history. Use `lockbox <path> doctor compact` outside foreign library calls. |
| New `LockboxInspector::verify_storage()` | Rust/CLI only. Deep physical ownership and free-zero validation; CLI `doctor --deep`. |
| `TransactionRecoveryPhase` gains `Rollback` and `Truncate` | Rust recovery orchestration only; Rust consumers with exhaustive matches must handle the new variants. |
| Existing `Lockbox::abort()` gains durable physical rollback | Rust only. Not an undo of a published commit. |
| Form record/value types gain `PartialEq` and `Eq` | Additive Rust comparison traits used by verification; serialized binding models are unchanged. |
| `format_version()` now reports 4 | Rust inspection behavior changes with the new native format; older containers require migration. |
| New owner fingerprint/lookup and migration comparison helpers | Rust migration orchestration only; not foreign key-management methods. |

The shared binding exclusions explicitly record these boundaries. Foreign writable file opens inherit Rust's automatic rollback, cleanup and truncation. Explicit read-only opens remain non-mutating and may report recovery required. File-backed operations and recovery stay inside Rust; do not emulate compaction by exporting bytes and overwriting the archive from a binding.

A commit error can occur after the new state is published. Reopen and inspect persisted contents before retrying an operation; do not assume every error means rollback. See [Transactions and recovery](../transactions.md) for the publication boundary and phase details.

Before releasing each language package, rebuild the native/WASM carrier, run the shared binding contract checks and its executable conformance suite, and verify its file-backed lifecycle and read-only behavior with the matching runtime. Passing the shared contract check alone does not establish that every packaged runtime has been rebuilt or tested.
