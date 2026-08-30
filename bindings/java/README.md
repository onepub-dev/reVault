# reVault for Java

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

The Java 22+ package provides Lockbox, Vault, key, and session agent classes. It
uses the Foreign Function and Memory API and includes the matching reVault
engine.

## Installation and native runtime

```kotlin
implementation("dev.onepub:revault-api:0.3.11")
```

The Maven JAR stores target libraries under `META-INF/native`. `Revault.load()`
selects the current operating system and architecture, extracts the matching
library to a temporary directory, and loads it through the Foreign Function
and Memory API. Run the application with native access enabled for its module.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```java
import java.nio.file.Path;
import static java.nio.charset.StandardCharsets.UTF_8;

var runtime = Revault.load();
try (var signing = runtime.generateProfileSigningKeyPair();
     var publicSigningKey = signing.publicKey();
     var vault = Vault.open(vaultPassphrase);
     var box = vault.openLockboxWithPassword(Path.of("team.lbox"), lockboxPassword)) {
  box.setOwnerSigningKey(signing); // Profile becomes this Lockbox's owner
  box.addFile("/hello.txt", "hello\n".getBytes(UTF_8), false);
  box.setVariable("owner", "alice");
  box.setSecretVariable("token", "secret".getBytes(UTF_8));
  int size = box.withSecretVariable("token", token -> token.length);
  box.commit();
}
```

`Revault.load()` only loads the installed native library. `Vault.open()` opens
the persistent encrypted local store; it never contacts the agent. A
`vaultPassphrase` opens the Vault and a `lockboxPassword` unlocks that archive;
callers own both byte arrays and should wipe them after use. Native failures
are thrown as `RevaultException`. `vault.agentSession()` is explicit: its
`closeLockbox`/`closeAll` operations forget temporary cached content keys and
do not delete lockbox files or persistent credentials.

`Revault.load(Path)` or `Revault.load(String)` loads a library supplied by the
application instead. Otherwise, reVault checks `REVAULT_LIBRARY` before using
the library in the JAR. A bare name uses the operating system library search
path. Owned objects are `AutoCloseable`; secret callback arrays are cleared
after the callback returns.

## Create, open, and replace

`Vault.open` and Lockbox open methods require existing resources. They do not
create missing state. Use open or create only when creation is acceptable and
use replacement methods only after an explicit destructive choice. A Lockbox
`commit` persists pending changes; `close` releases the handle and content
key held by this process without deleting the archive.

Java can open that archive using a Lockbox password, a profile key, or a
credential resolved from the Vault. A profile signing key becomes the Lockbox
owner key only when `setOwnerSigningKey` assigns that role.

## Secrets, exceptions, and ownership

Vault passphrases, Lockbox passwords, and 32-byte content keys are distinct
byte arrays owned by the caller. Clear them in `finally` blocks. Avoid converting
them to `String`, whose immutable storage cannot be erased.

Every handle that owns native resources is `AutoCloseable`; use try-with-resources as soon
as a constructor succeeds. Secret callbacks receive a temporary array that is
cleared after return and must not be captured or retained. Native failures
throw `RevaultException` with structured details and recovery guidance.

## Optional session agent

Ordinary Vault and Lockbox operations never start or contact the agent. Use
`AgentSession` when Lockbox keys need to be shared across processes or remain
available after the process that opened the Lockbox exits. `closeLockbox` and
`closeAll` forget agent cache entries only; they do
not delete files or persistent Vault credentials.

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

The package Javadocs document the classes and methods for the installed
version. The [Java conformance program](../e2e/java/Conformance.java) provides
complete worked workflows with cleanup and exception handling, and the
[example index](../API_EXAMPLES.md) tracks method coverage. Missing or
placeholder Javadocs are binding defects.
