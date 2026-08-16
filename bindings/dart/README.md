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

The package exposes one Dart library: `package:revault_api/revault_api.dart`.
Implementation files under `lib/src` are not separate public entry points.

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

final box = Lockbox.openFromAgent('/secrets/team.lbox');
try {
  // Use the process-local Lockbox handle.
} finally {
  box.close();
}

// Explicitly remove the independent key retained by the agent.
agent.closeLockbox('/secrets/team.lbox');
```

The agent does not keep a file handle open. “Open” means that it temporarily
holds the content key needed to reopen that lockbox.

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

See [UPGRADING.md](UPGRADING.md) when migrating from 0.2.x. See the
[repository documentation](https://github.com/onepub-dev/reVault/tree/main/docs)
for the file format, key management, and security model.

`REVAULT_LIBRARY` is a development-only override for native-library discovery.
