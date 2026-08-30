# Opening a reVault Vault after `sudo`

On Linux, a process started with `sudo` normally runs with root's effective
identity and environment. A Vault passphrase remembered by Auto Open is stored
in that desktop user's platform credential store. Linux normally provides that
store through Secret Service over the user's D-Bus session. Running as root
therefore changes both the identity used to access the store and the D-Bus
session used to reach it.

Do not copy the user's Vault passphrase into root's credential store. Run the
reVault operation as the original user, or restore that user's identity and
session context before opening the Vault.

## Prefer launching the operation as the user

When the privileged portion of a script can be separated from Vault access,
launch the Vault operation as the invoking user:

```console
sudo --user "$SUDO_USER" \
  DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$SUDO_UID/bus" \
  XDG_RUNTIME_DIR="/run/user/$SUDO_UID" \
  my-revault-application
```

The application can then use the default Vault normally:

```dart
await Revault.load();
final vault = Vault.open();
```

## Restore the user context inside a Dart process

Applications that deliberately start with privileges should release them
before accessing user-owned files or credentials. With `dcli`, capture the
invoking user's details first and then release privileges:

```dart
import 'dart:io';

import 'package:dcli/dcli.dart';
import 'package:revault_api/revault_api.dart';

Future<void> openInvokingUsersVault() async {
  final uidText = Platform.environment['SUDO_UID'];
  if (uidText == null) {
    throw StateError('This operation expects to have been launched by sudo.');
  }

  final uid = int.parse(uidText);
  final userHome = Shell.current.loggedInUsersHome;
  final runtimeDirectory = '/run/user/$uid';
  final sessionBusAddress = 'unix:path=$runtimeDirectory/bus';

  Shell.current.releasePrivileges();

  await Revault.load();
  final vault = Vault.open(
    pathTo: '$userHome/.local/share/lockbox/vault',
    platformCredentialContext: PlatformCredentialContext.linux(
      sessionBusAddress: sessionBusAddress,
    ),
  );

  try {
    // Use the invoking user's Vault.
  } finally {
    vault.close();
  }
}
```

`pathTo` is the directory containing `local-vault.lbox`. It also identifies the
credential store entry, so it must match the path used when the passphrase was
remembered. `PlatformCredentialContext.linux` passes the selected D-Bus address
directly to native code; reVault does not modify `DBUS_SESSION_BUS_ADDRESS` or
other process environment variables.

Release group privileges before user privileges, initialize the user's
supplementary groups, and ensure files are created with the user's ownership.
`Shell.current.releasePrivileges()` handles those identity changes. If an
application implements privilege changes directly, test the real, effective,
and saved IDs carefully and avoid retaining an unintended route back to root.

## Troubleshooting

- Confirm `/run/user/<uid>/bus` exists for the invoking user's active session.
- Confirm the process has released root's effective UID and GID before opening
  user-owned Vault files.
- Confirm `pathTo` points to the same Vault directory used when the passphrase
  was remembered.
- Treat an unavailable platform credential store as an authentication failure.
  On Linux, check that the user's Secret Service provider is reachable through
  the selected D-Bus session. Prompt for the Vault passphrase and hold it in a
  `SecretString` rather than silently using root's credentials.
