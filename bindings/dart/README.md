# reVault for Dart

reVault creates portable archives that are encrypted, compressed and signed.

The reVault archives are called 'Lockboxes'.

You can think of a Lockbox as a zip file on steriods. 

A Lockbox can be used to store:
* files
* directories
* symlinks
* file/directory permissions
* variables
* forms (collection of variables)

`revault_api` allows you to create/read/update reVault archives and manages the user's persistent
local Vault. It supports Linux, macOS, and Windows on x86-64 and ARM64.

The reVault Vault stores the keys to your Lockboxes as well as a Form definitions 
and a Contact list of other parties that you can securely share Lockboxes with.

Integration with your OS platform's secure store saves you from entering a 
password everytime you need to access a Lockbox.

Encryption is post quantum hybrid  (meaning it secure now and into the future)
and uses zstd for compression.

You can read/write/seek to the content of files stored in a Lockbox without
expanding its content to disk. 

Variables and form fields marked as 'secret' are protected from being swapped 
out to disk even if your laptop is suspended (subject to OS support).

reVault also provides CLI tooling to managed Lockboxes from the terminal. 

reVautl is written in Rust ensuring excellent performance with this Dart
API provides Dart native a wrapper for the Rust implementaiton.


# Sponsors

Fixed is sponsored by OnePub, the Dart private package repository.

<a href="https://onepub.dev">
  <img src="https://raw.githubusercontent.com/onepub-dev/reVault/main/images/LogoAndByLine.png" alt="OnePub" width="300">
</a>


```yaml
dependencies:
  revault_api: ^0.3.1
```

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

Flutter desktop does not support `Isolate.resolvePackageUri`. Pass the native
library path supplied by your application packaging layer when loading reVault:

```dart
await Revault.load(
  nativeLibraryPath: '/path/to/librevault_api.so',
);
```

On Linux the path identifies `librevault_api.so`. This explicit path takes
precedence over the `REVAULT_LIBRARY` environment variable and automatic Dart
package or executable-relative discovery.

Command-line launchers can select the same library without changing Dart code:

```console
REVAULT_LIBRARY=/opt/revault/lib/librevault_api.so dart run
```

## The four API concepts

- `Revault` loads the native runtime. It is not a vault.
- `Vault` is the persistent encrypted store for profiles, private keys,
  contacts, form definitions, signing keys, and remembered lockbox metadata.
- `Lockbox` is a portable encrypted `.lbox` archive.
- `AgentSession` controls the optional, session-agent process and its
  temporary cache of decrypted lockbox content keys. Use the SessionAgent
  when building processes that start and stop (like CLI tooling) and you want
  convienency/speed when re-opening a lockbox.


Passwords and passphrases use `SecretString`; binary keys use `SecretBytes`.
Both own wipeable byte storage and must be closed. `SecretString.fromString`
cannot erase the immutable Dart `String` used to create it, so password-input
adapters should return owned UTF-8 bytes for `SecretString.takeUtf8` where
possible.

Ordinary `Lockbox.open` calls do not require the Session Agent to be running
and will not start it.

Use `AgentSession` explicitly for CLI-style or multi-process workflows:

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
credential store. On platforms without MFA^*1^ enforcement, any
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

Future releases are planned to support MFA or equivalent user
presence whenever a vault or lockbox credential is retrieved to open a
lockbox. Code must feature-detect that capability when introduced.

See [UPGRADING.md](UPGRADING.md) when migrating from 0.2.x. See the
[repository documentation](https://github.com/onepub-dev/reVault/tree/main/docs)
for the file format, key management, and security model.

`REVAULT_LIBRARY` is a process-wide native-library discovery override. Prefer
the `nativeLibraryPath` argument when application code already knows the bundle
location; use the environment variable when a launcher or test harness controls
the process.


*1 Multifactore Auth is a planned feature.
