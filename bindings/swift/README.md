# reVault for Swift

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. SwiftPM provides a native `RevaultAPI` product for
macOS and Linux. See the
[reVault manual](https://docs.revault.onepub.dev/).

```swift
.package(url: "https://github.com/onepub-dev/revault-swift", exact: "0.3.11")
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```swift
let runtime = Revault()
let signing = try runtime.generateProfileSigningKeyPair()
let publicSigningKey = try signing.publicKey()
let box = try runtime.lockboxCreate(Data(repeating: 0, count: 32))
try box.setOwnerSigningKey(signing) // Profile becomes this Lockbox's owner
try box.addFile("/hello.txt", Data("hello\n".utf8), false)
try box.setVariable("owner", "alice")
try box.setSecretVariable("token", Data("secret".utf8))
try box.withSecretVariable("token") { token in token.count }
try box.commit()
try box.free()
try publicSigningKey.dispose()
try signing.dispose()
```

`Revault()` loads the installed runtime facade; loading does not open a Vault or
Lockbox. A Vault passphrase, Lockbox password, and content key are distinct
caller-owned secrets. Native failures are thrown as `RevaultError`. Agent use
is explicit via `AgentSession`; closing an entry forgets only temporary cached
content keys. Secret callbacks receive a temporary raw buffer that is zeroed
after return; do not convert it to a retained `String` or `Data`.
