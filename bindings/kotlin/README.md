# reVault for Kotlin

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This package supplies idiomatic Kotlin aliases over the
complete Java 22+ runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```kotlin
implementation("dev.onepub:revault-api-kotlin:0.2.0")
```

```kotlin
val vault = Vault()
vault.createLockbox(ByteArray(32)).use { box -> // load a real key securely
    box.addFile("/hello.txt", "hello\n".encodeToByteArray(), false)
    box.setVariable("owner", "alice")
    box.setSecretVariable("token", "secret".encodeToByteArray())
    box.withSecretVariable("token") { token -> token.size }
    box.commit()
}
```

Owned objects must be closed. Secret callback arrays are temporary and cleared
after the callback returns.
