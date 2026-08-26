# reVault for Dart

`revault_api` provides typed Dart bindings for creating and opening encrypted
reVault Lockbox archives and managing the user's persistent local Vault. The
native core is implemented in Rust, while the public API uses Dart classes and
explicitly owned secret values.

The Dart package supports Dart and Flutter desktop applications on Linux,
macOS, and Windows, on both x86-64 and ARM64. It does not currently support
Dart web applications: this binding uses `dart:ffi` and `dart:io`. Projects
targeting browsers or WebAssembly should use the
[JavaScript/WASM package](https://github.com/onepub-dev/reVault/tree/main/bindings/wasm).

## Installation

Add the current package from pub.dev:

```console
dart pub add revault_api
```

Then load the bundled native runtime once during application startup:

```dart
import 'package:revault_api/revault_api.dart';

Future<void> main() async {
  await Revault.load();
  // Vault, Lockbox, and AgentSession are now ready to use.
}
```

The current release is `0.3.11`. Normal applications do not need to download a
native library or set an environment variable; the package build hook bundles
the correct native carrier.

## Create a Vault and Lockbox in Dart

The following setup creates or opens a Vault, creates a password-protected
Lockbox, and stores the Lockbox password inside the encrypted Vault. It uses
`SecretString.fromString` to keep the example self-contained. A production
password prompt should supply owned UTF-8 bytes to `SecretString.takeUtf8` so
the original input buffer can also be wiped.

```dart
import 'dart:io';

import 'package:revault_api/revault_api.dart';

Future<void> main() async {
  await Revault.load();

  final workspace = Directory('.revault-example')..createSync();
  final vaultPassphrase = SecretString.fromString('replace-this-passphrase');
  final lockboxPassword = SecretString.fromString('replace-this-password');
  final vault = Vault.openOrCreate(
    pathTo: '${workspace.path}/vault',
    passphrase: vaultPassphrase,
  );

  try {
    final path = '${workspace.path}/team-secrets.lbox';
    final lockbox = File(path).existsSync()
        ? Lockbox.open(path, password: lockboxPassword)
        : Lockbox.create(path, password: lockboxPassword);
    try {
      lockbox.setDescription('Team deployment credentials');
      lockbox.commit();
      vault.rememberPassword(lockbox.id, lockboxPassword);
    } finally {
      lockbox.close();
    }
  } finally {
    vault.close();
    lockboxPassword.close();
    vaultPassphrase.close();
  }
}
```

A complete runnable version with interactive password prompts is included in
[`example/getting_started.dart`](example/getting_started.dart).

## Create a Vault and Lockbox with the CLI

The reVault CLI and Dart binding use the same Vault and Lockbox formats. Install
the CLI, initialize the default Vault and profile, and create a Lockbox for that
profile:

```console
cargo install revault_cli
lbx vault init
lbx team-secrets.lbox create \
  --description 'Team deployment credentials'
```

The CLI prints recovery material during Vault initialization. Store that
material securely: losing both the Vault and its recovery material can make
profile-protected Lockboxes unrecoverable.

The Dart application can then open the CLI-created Vault and Lockbox using the
Vault passphrase:

```dart
import 'package:revault_api/revault_api.dart';

Future<void> openExisting(SecretString vaultPassphrase) async {
  await Revault.load();
  final vault = Vault.open(passphrase: vaultPassphrase);
  try {
    final lockbox = Lockbox.open('team-secrets.lbox', vault: vault);
    try {
      print(lockbox.description);
      print(lockbox.list('/', recursive: true));
    } finally {
      lockbox.close();
    }
  } finally {
    vault.close();
  }
}
```

## Core API concepts

- `Revault` loads the process-wide native runtime and provides key-generation,
  import, export, and format utilities.
- `Vault` is the persistent encrypted store for profiles, private keys,
  contacts, signing keys, and remembered Lockbox credentials and metadata.
- `Lockbox` is a portable encrypted `.lbox` archive containing files,
  variables, secrets, and structured forms.
- `AgentSession` controls the optional session-agent process and its temporary
  cache of decrypted Lockbox content keys.

Passwords and passphrases use `SecretString`; binary keys use `SecretBytes`.
Both own wipeable byte storage and must be closed. `SecretString.fromString`
cannot erase the immutable Dart `String` used to create it, so password-input
adapters should return owned UTF-8 bytes for `SecretString.takeUtf8` whenever
possible.

## Use the optional session agent

Ordinary `Lockbox.open` calls are process-local and never start or contact the
agent. Use `AgentSession` explicitly for CLI-style or multi-process workflows:

```dart
final agent = AgentSession.instance;
agent.start();
agent.keepOpenWithPassword(
  'team-secrets.lbox',
  lockboxPassword,
  duration: const Duration(minutes: 30),
);

final lockbox = agent.acquireOpenLockbox('team-secrets.lbox');
try {
  print(lockbox.list('/', recursive: true));
} finally {
  lockbox.close();
}

agent.closeLockbox('team-secrets.lbox');
```

The agent stores a temporary content key, not an open file handle. Acquiring a
Lockbox does not extend the agent entry's lifetime. The returned process-local
handle owns an independent key and remains usable after agent expiry until it
is closed.

## Lockbox descriptions

A Lockbox can carry a human-readable description of its purpose. The text is
stored inside the encrypted archive rather than its public header, so it can be
read only after the Lockbox is opened.

```dart
lockbox.setDescription(
  'Production deployment credentials and recovery material',
);
lockbox.commit();
print(lockbox.description);

lockbox.clearDescription();
lockbox.commit();
```

## Platform credentials and unattended access

`Vault.rememberPassphrase` stores the Vault passphrase in the operating-system
credential store. On platforms without per-use user-presence enforcement, any
process able to access that user's platform credentials may be able to retrieve
it. This grants unattended access to the Vault and every Lockbox for which the
Vault contains a usable credential.

Remembered Lockbox passwords remain encrypted inside the Vault; they are not
stored as independent operating-system credentials. With no explicit
credential, `Lockbox.open(path)` opens the default Vault using its
platform-stored passphrase and asks the Vault for a matching Lockbox password
or profile key. It does not persist a raw content key or contact the session
agent.

Agent expiry and `AgentSession.closeAll()` are therefore not authentication
boundaries while the Vault passphrase remains retrievable without interaction.
The agent provides stronger isolation when the user unlocks the Vault
interactively, retains only selected Lockbox keys in the agent, and then closes
the Vault.

For services launched with `sudo`, see
[Opening a reVault Vault after sudo](https://github.com/onepub-dev/reVault/blob/main/docs/opening_a_vault_after_sudo.md).

## Native library loading

`Revault.load()` normally loads the target-specific carrier supplied by the
package. An application installer that deliberately maintains one shared
carrier can provide its path explicitly:

```dart
await Revault.load(
  nativeLibraryPath: '/opt/my_app/lib/librevault_api.so',
);
```

Resolution order is an explicit path, a non-empty inherited
`REVAULT_LIBRARY`, and then the package carrier. A library name rather than a
path delegates lookup to the operating system's normal library search rules.

## More information

- See [`UPGRADING.md`](UPGRADING.md) when migrating from `0.2.x`.
- Browse the [complete API example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md).
- Read the [reVault manual](https://docs.revault.onepub.dev/) for user guides,
  file-format concepts, and the security model.
- See the [binding contribution and release guide](https://github.com/onepub-dev/reVault/blob/main/bindings/CONTRIBUTING.md) before
  changing generated APIs or publication packaging.
- Report problems in the [reVault issue tracker](https://github.com/onepub-dev/reVault/issues).
