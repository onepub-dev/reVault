# reVault for Swift

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. SwiftPM provides a native `RevaultAPI` product for
macOS and Linux. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```swift
.package(url: "https://github.com/onepub-dev/revault-api", exact: "0.2.0")
```

```swift
let vault = Vault()
let box = try vault.lockboxCreate(Data(repeating: 0, count: 32))
try box.addFile("/hello.txt", Data("hello\n".utf8), false)
try box.setVariable("owner", "alice")
try box.setSecretVariable("token", Data("secret".utf8))
try box.withSecretVariable("token") { token in token.count }
try box.commit()
try box.free()
```

Secret callbacks receive a temporary raw buffer that is zeroed after return.
Do not convert it to a retained `String` or `Data`.
