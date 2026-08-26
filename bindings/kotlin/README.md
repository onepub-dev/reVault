# reVault for Kotlin

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This package supplies idiomatic Kotlin aliases over the
complete Java 22+ runtime. See the
[reVault manual](https://docs.revault.onepub.dev/).

```kotlin
implementation("dev.onepub:revault-api-kotlin:0.3.11")
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```kotlin
val runtime = Revault.load()
val signing = runtime.generateProfileSigningKeyPair()
val publicSigningKey = signing.publicKey()
val vault = Vault.open(vaultPassphrase) // vaultPassphrase is caller-owned
vault.openLockboxWithPassword(path, lockboxPassword).use { box ->
    box.setOwnerSigningKey(signing) // Profile becomes this Lockbox's owner
    box.addFile("/hello.txt", "hello\n".encodeToByteArray(), false)
    box.setVariable("owner", "alice")
    box.setSecretVariable("token", "secret".encodeToByteArray())
    box.withSecretVariable("token") { token -> token.size }
    box.commit()
}
publicSigningKey.close()
signing.close()
```

`Revault.load()` loads the installed carrier but does not open a Vault or
Lockbox. `Vault.open` and `Lockbox.open` are non-creating operations. The
vault passphrase, lockbox password, and content key are distinct secrets;
callers own their `ByteArray`s and should clear them as soon as the operation
returns. Native failures are typed Java/Kotlin exceptions. Agent use is
explicit through `vault.agentSession()` and caches only temporary content keys.
Owned handles must be closed; secret callback arrays are cleared after return.

`Revault.load(Path)` or `Revault.load(String)` selects an application-owned
carrier. Otherwise a non-empty inherited `REVAULT_LIBRARY` is used before the
JAR carrier; a bare string delegates to the operating-system search path.
