# reVault for Kotlin

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This package supplies idiomatic Kotlin aliases over the
complete Java 22+ runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```kotlin
implementation("dev.onepub:revault-api-kotlin:0.2.0")
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```kotlin
val runtime = Revault.load()
val vault = Vault.open(vaultPassphrase) // vaultPassphrase is caller-owned
vault.openLockboxWithPassword(path, lockboxPassword).use { box ->
    box.addFile("/hello.txt", "hello\n".encodeToByteArray(), false)
    box.setVariable("owner", "alice")
    box.setSecretVariable("token", "secret".encodeToByteArray())
    box.withSecretVariable("token") { token -> token.size }
    box.commit()
}
```

`Revault.load()` loads the installed carrier but does not open a Vault or
Lockbox. `Vault.open` and `Lockbox.open` are non-creating operations. The
vault passphrase, lockbox password, and content key are distinct secrets;
callers own their `ByteArray`s and should clear them as soon as the operation
returns. Native failures are typed Java/Kotlin exceptions. Agent use is
explicit through `vault.agentSession()` and caches only temporary content keys.
Owned handles must be closed; secret callback arrays are cleared after return.
