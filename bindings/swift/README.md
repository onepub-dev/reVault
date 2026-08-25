# reVault for Swift

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. SwiftPM provides a native `RevaultAPI` product for
macOS and Linux. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```swift
.package(url: "https://github.com/onepub-dev/revault-api", exact: "0.2.0")
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

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
Lockbox. A vault passphrase, lockbox password, and content key are distinct
caller-owned secrets. Native failures are thrown as `RevaultError`. Agent use
is explicit via `AgentSession`; closing an entry forgets only temporary cached
content keys. Secret callbacks receive a temporary raw buffer that is zeroed
after return; do not convert it to a retained `String` or `Data`.
