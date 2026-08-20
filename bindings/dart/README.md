# reVault for Dart

`revault_api` opens encrypted Lockbox archives and manages the user's persistent
local Vault. It supports Linux, macOS, and Windows on x86-64 and ARM64.

```yaml
dependencies:
  revault_api: ^0.3.0
```

The complete method-example index is in [`../API_EXAMPLES.md`](../API_EXAMPLES.md).

Load the native runtime once, then work with the domain objects directly:

```dart
import 'package:revault_api/revault_api.dart';

Future<void> main() async {
  await Revault.load();

  final vaultPassphrase = SecretString.takeUtf8(await prompt.readUtf8());
  final vault = Vault.open(passphrase: vaultPassphrase);

  try {
    final lockbox = Lockbox.open('/secrets/team.lbox', vault: vault);
    try {
      print(lockbox.list('/', recursive: true));
    } finally {
      lockbox.close();
    }
  } finally {
    vault.close();
    vaultPassphrase.close();
  }
}
```

## The four API concepts

- `Revault` loads the native runtime. It is not a vault.
- `Vault` is the persistent encrypted store for profiles, private keys,
  contacts, signing keys, and remembered lockbox metadata.
- `Lockbox` is a portable encrypted `.lbox` archive.
- `AgentSession` controls the optional, single session-agent process and its
  temporary cache of decrypted lockbox content keys.

Passwords and passphrases use `SecretString`; binary keys use `SecretBytes`.
Both own wipeable byte storage and must be closed. `SecretString.fromString`
cannot erase the immutable Dart `String` used to create it, so password-input
adapters should return owned UTF-8 bytes for `SecretString.takeUtf8` where
possible.

Ordinary `Lockbox.open` calls are process-local and never start or contact the
agent. Use `AgentSession` explicitly for CLI-style or multi-process workflows:

```dart
final agent = AgentSession.instance;
agent.start();
agent.keepOpenWithPassword(
  '/secrets/team.lbox',
  lockboxPassword,
  duration: const Duration(minutes: 30),
);

final box = agent.acquireOpenLockbox('/secrets/team.lbox');
try {
  // Use the process-local Lockbox handle.
} finally {
  box.close();
}

// Explicitly remove the independent key retained by the agent.
agent.closeLockbox('/secrets/team.lbox');
```

The agent does not keep a file handle open. “Open” means that it temporarily
holds the content key needed to reopen that lockbox. Acquiring a Lockbox does
not extend the agent TTL. The returned process-local handle owns a copy of the
content key and remains usable after agent expiry until `close()` is called. A
native finalizer is a safety net for forgotten handles, but deterministic
`close()` remains the preferred way to wipe the process-local key promptly.

## Lockbox descriptions

A Lockbox can carry a human-readable description of its purpose. The text is
stored inside the encrypted archive, not its public header, and therefore can
only be read after the Lockbox is opened. It accepts the same UTF-8 content and
one-mebibyte limit as a normal variable value.

```dart
lockbox.setDescription(
  'Production deployment credentials and recovery material',
);
lockbox.commit();
print(lockbox.description);
```

Use `clearDescription()` followed by `commit()` to remove it.

## Platform credentials and unattended access

`Vault.rememberPassphrase` stores the vault passphrase in the operating system
credential store. On platforms without per-use user-presence enforcement, any
process able to access that user's platform credentials may retrieve it. That
provides unattended access to the vault and every lockbox for which the vault
contains a usable credential.

Remembered lockbox passwords remain encrypted inside the Vault; they are not
stored as independent operating-system credentials. Consequently,
`Lockbox.open(path)` first opens the default Vault using its platform-stored
passphrase, then asks the Vault for the applicable lockbox password or profile
key. It never persists a raw content key or contacts the session agent.

Consequently, agent expiry and `AgentSession.closeAll()` are not authentication
boundaries while the vault passphrase remains retrievable without interaction.
The agent provides its strongest isolation when the user unlocks the vault
interactively, retains only selected lockbox keys in the agent, and then closes
the vault.

Future releases may support platform-enforced biometric or equivalent user
presence whenever a vault or lockbox credential is retrieved to open a
lockbox. Code must feature-detect that capability when introduced.

### Opening a Vault after sudo

On Linux, a process launched by `sudo` normally inherits root's effective
identity and environment. The user's remembered Vault passphrase is in that
user's Secret Service session, so both must be restored deliberately:

```dart
final invokingUid = int.parse(Platform.environment['SUDO_UID']!);
final invokingGid = int.parse(Platform.environment['SUDO_GID']!);
final busAddress = 'unix:path=/run/user/$invokingUid/bus';

// Use the platform's setegid/seteuid equivalents before opening the Vault.
dropEffectivePrivileges(uid: invokingUid, gid: invokingGid);

final vault = Vault.open(
  pathTo: '/home/the-user/.local/share/lockbox/vault',
  platformCredentialContext: PlatformCredentialContext.linux(
    sessionBusAddress: busAddress,
  ),
);
```

`pathTo` is the directory containing `local-vault.lbox`. It also identifies
the credential-store item, so it must match the path used when the passphrase
was remembered. The session address is passed directly to native code; reVault
does not modify `DBUS_SESSION_BUS_ADDRESS` or any other process environment
variable.

Applications using dcli can use its sudo-user and privilege-release helpers to
restore the invoking user's effective uid/gid and session values, then pass the
resulting bus address to `PlatformCredentialContext.linux`. dcli does not need
to know anything about reVault or its agent.

See [UPGRADING.md](UPGRADING.md) when migrating from 0.2.x. See the
[repository documentation](https://github.com/onepub-dev/reVault/tree/main/docs)
for the file format, key management, and security model.

The package build hook publishes the target-specific Revault carrier as a
native code asset. `Revault.load()` therefore needs no library path or
environment variable; `dart build cli` and Flutter builds bundle the carrier
automatically.

Before publishing with `pub_release`, stage all six prebuilt carriers:

```console
REVAULT_DART_NATIVE_SOURCE=/path/to/dart/lib/src/native \
  dart tool/pre_release_hook/stage_native_assets.dart 0.3.5
```

`pub_release` discovers scripts under `tool/pre_release_hook/` and supplies
the version argument automatically. Its dry run passes `--dry-run`; the hook
then validates the six carriers without copying them. The existing release
assembler can be used as the source with
`REVAULT_DART_NATIVE_SOURCE=../../packages/dart/lib/src/native`.
