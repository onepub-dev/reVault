# reVault for Dart

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. The Dart package provides an owned, class-based API and
generated Protobuf result types. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```yaml
dependencies:
  revault_api: ^0.2.0
```

```dart
final vault = await Vault.load();
final box = vault.createLockbox(Uint8List(32)); // load a real key securely
box.addFile(
  '/hello.txt',
  Uint8List.fromList('hello\n'.codeUnits),
  replace: false,
);
box.setVariable('owner', 'alice');
box.setSecretVariable('token', Uint8List.fromList('secret'.codeUnits));
box.withSecretVariable('token', (token) => token.length);
box.commit();
box.dispose();
```

Packaged runtimes support Linux, macOS, and Windows on x86-64 and ARM64.
`REVAULT_LIBRARY` is a development-only override for native discovery.
