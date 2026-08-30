# reVault for Kotlin

reVault is a fast, local toolkit for creating secure portable archives called
Lockboxes. Each Lockbox is encrypted, compressed, and signed. It can store
files and directory trees, variables such as API keys, and forms such as login
details.

Lockboxes are easy to copy, share, and back up, and they do not require a
hosted service. The engine is designed for speed and effective compression.
Applications can read, write, and seek within stored files without extracting
the archive, and recover data from partial corruption. reVault provides a
command line tool for everyday work and APIs for application code.

Read the [reVault manual](https://docs.revault.onepub.dev/) for the quick start,
core concepts, and security model.

Your Vault holds your profile and contacts. The CLI protects a new Lockbox for
your profile by default, and you can grant access to contacts using their
public keys. Use password access when you do not have a recipient's contact
(public key) details.

This package supplies Kotlin APIs for Lockboxes, Vaults, keys, and the session
agent over the complete Java 22+ runtime.

## Installation and native runtime

```kotlin
implementation("dev.onepub:revault-api-kotlin:0.3.11")
```

The Kotlin artifact uses the Java runtime package. Its JAR stores target
libraries under `META-INF/native`; `Revault.load()` selects and extracts the
library for the current operating system and architecture. Run the application
with native access enabled for its module.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```kotlin
val runtime = Revault.load()
val signing = runtime.generateProfileSigningKeyPair()
val publicSigningKey = signing.publicKey()
val vault = Vault.open(vaultPassphrase) // vaultPassphrase is owned by the caller
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

`Revault.load()` loads the installed native library but does not open a Vault or
Lockbox. `Vault.open` and `Lockbox.open` are non-creating operations. The
Vault passphrase, Lockbox password, and content key are distinct secrets;
callers own their `ByteArray`s and should clear them as soon as the operation
returns. Native failures are typed Java/Kotlin exceptions. Agent use is
explicit through `vault.agentSession()` and caches only temporary content keys.
Owned handles must be closed; secret callback arrays are cleared after return.

`Revault.load(Path)` or `Revault.load(String)` loads a native library supplied
by the application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the
library included in the Java runtime JAR. A bare name uses the operating system
search path.

## Create, open, and replace

`Vault.open` and Lockbox open methods require existing resources and never
create them. Use open or create only when either outcome is acceptable and use
replace only after an explicit destructive decision. `commit` persists
pending Lockbox changes; `close` releases the handle and content key held by
this process but does not delete the archive.

Open the resulting archive with a Lockbox password, profile key, or credential
resolved from the Vault. A profile signing key has the owner role only after
`setOwnerSigningKey` assigns it to a Lockbox.

## Secrets, exceptions, and ownership

Vault passphrases, Lockbox passwords, and 32-byte content keys are separate
`ByteArray` values owned by the caller. Clear them in `finally`. Do not convert them
to immutable `String` values. Secret callbacks receive a temporary array that
is cleared after return and must not escape the lambda.

Use `use` for every `AutoCloseable` Vault, Lockbox, key, and agent handle.
Native failures are typed exceptions with structured details rather than
global error state.

## Optional session agent

Ordinary Lockbox operations never start or consult the agent. Use
`AgentSession` when Lockbox keys need to be shared across processes or remain
available after the process that opened the Lockbox exits.
`closeLockbox` and `closeAll` forget cached keys; they do not delete Lockbox
files or credentials stored in the Vault.

## Platform credential store

The operating system credential store can hold the Vault passphrase. The
user's operating system login normally unlocks that store. After login,
another process running as that user may be able to retrieve the passphrase if
the access policy applied to the saved Vault passphrase does not require
approval for each retrieval. Exact access depends on the operating system, the
credential store configuration, and that access policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault
can then provide access to Lockboxes through profile keys or remembered
Lockbox passwords. Both remain encrypted inside the Vault; they are not copied
to the operating system credential store.

Agent expiry improves memory hygiene. It is not an authentication boundary
after login if the saved Vault passphrase can be retrieved without approval.

## API documentation and support

KDoc and the underlying Java Javadocs describe the installed package version.
The [Kotlin conformance program](../e2e/kotlin/KotlinConformance.kt) supplies
complete worked examples with cleanup and typed failures. The
[example index](../API_EXAMPLES.md) tracks method coverage.
