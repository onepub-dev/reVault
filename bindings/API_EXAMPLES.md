# Language API worked examples

Every public operation in a language facade must have a worked example. The
examples are intentionally kept next to the language API, while the conformance
runners provide executable coverage of the same operations.

## What counts as an example

An example must show why the operation is used, provide realistic arguments,
and show the ownership and cleanup rules. For an operation that returns a
secret, key, or native resource, the example must also show the callback,
`close`, `dispose`, `free`, `defer`, or equivalent lifetime boundary. A method
signature, a sentence beginning with “Returns…”, or a package-install snippet
is not an example.

## Canonical workflow

The normal workflow is the same in every language:

1. Load the native runtime (`Revault.load`, a language-native constructor, or
   the package loader).
2. Open a `Vault` when persistent profiles, credentials, or remembered lockbox
   access are needed.
3. Open or create a `Lockbox`; the lockbox retains its content key until it is
   closed.
4. Perform the operation and commit mutations where the language API requires
   it.
5. Close/dispose the lockbox and vault. Use `AgentSession` only when a
   cooperating process needs an explicitly time-limited cached key.

For example, the Dart workflow is:

```dart
final revault = await Revault.load();
final vault = Vault.open(passphrase: vaultPassphrase);
final lockbox = Lockbox.open('/secrets/team.lbox', vault: vault);
try {
  lockbox.setVariable('owner', 'alice');
  lockbox.commit();
} finally {
  lockbox.close();
  vault.close();
}
```

Each other language uses the same workflow with its native lifetime idiom. The
package README for each language contains the short starting example; the
conformance source contains the complete operation examples:

| Language | Native API documentation | Executable examples |
| --- | --- | --- |
| C | [`bindings/c/README.md`](c/README.md) | [`bindings/e2e/c/conformance.c`](e2e/c/conformance.c) |
| C++ | [`bindings/cpp/README.md`](cpp/README.md) | [`bindings/e2e/cpp/conformance.cpp`](e2e/cpp/conformance.cpp) |
| C# | [`bindings/csharp/README.md`](csharp/README.md) | [`bindings/e2e/csharp/Program.cs`](e2e/csharp/Program.cs) |
| Dart | [`bindings/dart/README.md`](dart/README.md) | [`bindings/e2e/dart/conformance.dart`](e2e/dart/conformance.dart) |
| Go | [`bindings/go/README.md`](go/README.md) | [`bindings/e2e/go/conformance.go`](e2e/go/conformance.go) |
| Java | [`bindings/java/README.md`](java/README.md) | [`bindings/e2e/java/Conformance.java`](e2e/java/Conformance.java) |
| JavaScript | [`bindings/javascript/README.md`](javascript/README.md) | [`bindings/e2e/javascript/conformance.js`](e2e/javascript/conformance.js) |
| Kotlin | [`bindings/kotlin/README.md`](kotlin/README.md) | [`bindings/e2e/kotlin/KotlinConformance.kt`](e2e/kotlin/KotlinConformance.kt) |
| Lua | [`bindings/lua/README.md`](lua/README.md) | [`bindings/e2e/lua/conformance.lua`](e2e/lua/conformance.lua) |
| PHP | [`bindings/php/README.md`](php/README.md) | [`bindings/e2e/php/conformance.php`](e2e/php/conformance.php) |
| Python | [`bindings/python/README.md`](python/README.md) | [`bindings/e2e/python/conformance.py`](e2e/python/conformance.py) |
| Ruby | [`bindings/ruby/README.md`](ruby/README.md) | [`bindings/e2e/ruby/conformance.rb`](e2e/ruby/conformance.rb) |
| Rust | [`bindings/rust/README.md`](rust/README.md) | [`bindings/e2e/rust/src/main.rs`](e2e/rust/src/main.rs) |
| Swift | [`bindings/swift/README.md`](swift/README.md) | [`bindings/swift/Sources/RevaultConformance/main.swift`](swift/Sources/RevaultConformance/main.swift) |
| TypeScript | [`bindings/typescript/README.md`](typescript/README.md) | [`bindings/e2e/typescript/conformance.ts`](e2e/typescript/conformance.ts) |
| WASM | [`bindings/wasm/README.md`](wasm/README.md) | [`bindings/wasm/index.js`](wasm/index.js) and the hosted WASM package tests |

When a method is added or renamed, its native documentation and the matching
conformance example must be added in the same change. The bindings lint job
must reject a public method that has no documentation/example link. The
complete operation inventory is [`e2e/operations.tsv`](e2e/operations.tsv);
the conformance contract requires every row to be exercised with concrete
inputs and asserted outputs.
