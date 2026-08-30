# reVault for Dart

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

`revault_api` provides typed Dart classes for Lockboxes, the Vault, and the
optional session agent. The native core is implemented in Rust, while the
public API uses Dart domain objects and explicitly owned secret values.

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
the correct native library.

## Create a Vault and Lockbox in Dart

The following setup creates or opens a Vault, creates a Lockbox protected with
a password, and stores the Lockbox password inside the encrypted Vault. It uses
`SecretString.fromString` to keep the example complete on its own. A production
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

Install the reVault CLI, initialize the default Vault and profile, and create a
Lockbox for that profile:

```console
cargo install revault_cli
lbx vault init
lbx team-secrets.lbox create \
  --description 'Team deployment credentials'
```

The CLI prints recovery material during Vault initialization. Store that
material securely: losing both the Vault and its recovery material can make
Lockboxes protected by a profile unrecoverable.

The Dart application can then open the Vault and Lockbox created with the CLI
using the Vault passphrase:

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

- `Revault` loads the native runtime shared by the process and provides key generation,
  import, export, and format utilities.
- `Vault` is the persistent encrypted store for profiles, private keys,
  contacts, signing keys, and remembered Lockbox credentials and metadata.
- `Lockbox` is a portable encrypted `.lbox` archive containing files,
  variables, secrets, and structured forms.
- `AgentSession` controls the optional Session Agent process and its temporary
  cache of decrypted Lockbox content keys.

Passwords and passphrases use `SecretString`; binary keys use `SecretBytes`.
Both own wipeable byte storage and must be closed. `SecretString.fromString`
cannot erase the immutable Dart `String` used to create it, so password input
adapters should return owned UTF-8 bytes for `SecretString.takeUtf8` whenever
possible.

## Use the optional Session Agent

Ordinary `Lockbox.open` calls keep their state in this process and never start
or contact the agent. Use `AgentSession` when Lockbox keys need to be shared
across processes or remain available after the process that opened the Lockbox
exits:

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
Lockbox does not extend the agent entry's lifetime. The returned
handle owns an independent key and remains usable after agent expiry until it
is closed.

## Lockbox descriptions

A Lockbox can carry a readable description of its purpose. The text is
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

## Platform credential store

`Vault.rememberPassphrase` stores the Vault passphrase in the operating system
credential store. The user's operating system login normally unlocks that
store. After login, another process running as that user may be able to
retrieve the passphrase if the access policy applied to the saved Vault
passphrase does not require approval for each retrieval. Exact access depends
on the operating system, the credential store configuration, and that access
policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault can
then provide access to Lockboxes through profile keys or remembered Lockbox
passwords. Both remain encrypted inside the Vault; they are not copied to the
operating system credential store.

With no explicit credential, `Lockbox.open(path)` opens the default Vault using
the passphrase stored by the platform and asks the Vault for a matching profile
key or remembered Lockbox password. It does not contact the Session Agent.

Agent expiry and `AgentSession.closeAll()` are therefore not authentication
boundaries after login if the saved Vault passphrase can be retrieved without
approval.
The agent provides stronger isolation when the user unlocks the Vault
interactively, retains only selected Lockbox keys in the agent, and then closes
the Vault.

For services launched with `sudo`, see
[Opening a reVault Vault after sudo](https://github.com/onepub-dev/reVault/blob/main/docs/opening_a_vault_after_sudo.md).

## Native runtime distribution

The pub.dev package uses Dart native assets. Its build hook selects the library
for the application's operating system and architecture and bundles it into
Dart CLI and Flutter desktop builds. `Revault.load()` opens that bundled
library. An application that maintains its own copy can provide its path
explicitly:

```dart
await Revault.load(
  nativeLibraryPath: '/opt/my_app/lib/librevault_api.so',
);
```

Resolution order is an explicit path, a nonempty inherited
`REVAULT_LIBRARY`, and then the library bundled with the package. A library
name rather than a path uses the operating system's normal search rules.

## More information

- See [`UPGRADING.md`](UPGRADING.md) when migrating from `0.2.x`.
- Browse the [complete API example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md).
- Read the [reVault manual](https://docs.revault.onepub.dev/) for user guides,
  file format concepts, and the security model.
- See the [binding contribution and release guide](https://github.com/onepub-dev/reVault/blob/main/bindings/CONTRIBUTING.md) before
  changing generated APIs or publication packaging.
- Report problems in the [reVault issue tracker](https://github.com/onepub-dev/reVault/issues).
- Treat missing class or method documentation on [pub.dev](https://pub.dev/documentation/revault_api/latest/) as a binding defect; `dart doc` and the conformance inventory are release checks.
