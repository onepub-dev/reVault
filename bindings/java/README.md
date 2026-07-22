# reVault for Java

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Java 22+ package uses the Foreign Function & Memory
API and includes the matching native runtime. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```kotlin
implementation("dev.onepub:revault-api:0.2.0")
```

```java
import static java.nio.charset.StandardCharsets.UTF_8;

var vault = new Revault();
try (var box = vault.createLockbox(new byte[32])) { // load a real key securely
  box.addFile("/hello.txt", "hello\n".getBytes(UTF_8), false);
  box.setVariable("owner", "alice");
  box.setSecretVariable("token", "secret".getBytes(UTF_8));
  int size = box.withSecretVariable("token", token -> token.length);
  box.commit();
}
```

Run with native access enabled for this module/application. Owned objects are
`AutoCloseable`; secret callback arrays are cleared after the callback returns.
