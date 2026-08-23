# reVault for Java

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Java 22+ package uses the Foreign Function & Memory
API and includes the matching native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```kotlin
implementation("dev.onepub:revault-api:0.2.0")
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

```java
import java.nio.file.Path;
import static java.nio.charset.StandardCharsets.UTF_8;

try (var vault = Vault.open(vaultPassphrase);
     var box = vault.openLockboxWithPassword(Path.of("team.lbox"), lockboxPassword)) {
  box.addFile("/hello.txt", "hello\n".getBytes(UTF_8), false);
  box.setVariable("owner", "alice");
  box.setSecretVariable("token", "secret".getBytes(UTF_8));
  int size = box.withSecretVariable("token", token -> token.length);
  box.commit();
}
```

`Revault.load()` only loads the installed native carrier. `Vault.open()` opens
the persistent encrypted local store; it never contacts the agent. A
`vaultPassphrase` opens the Vault and a `lockboxPassword` unlocks that archive;
callers own both byte arrays and should wipe them after use. Native failures
are thrown as `RevaultException`. `vault.agentSession()` is explicit: its
`closeLockbox`/`closeAll` operations forget temporary cached content keys and
do not delete lockbox files or persistent credentials.

Run with native access enabled for this module/application. Owned objects are
`AutoCloseable`; secret callback arrays are cleared after the callback returns.
