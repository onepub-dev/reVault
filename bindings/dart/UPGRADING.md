# Upgrading revault_api from 0.2.x to 0.3.0

Version 0.3.0 deliberately breaks the Dart facade so its names describe the
security objects users actually operate. The native lockbox and vault formats
are unchanged; this is a source migration, not a data migration.

## Load the runtime

`Vault` was the 0.2 catch-all native facade. It is now `Revault`, and `Vault`
means the persistent local key and metadata store.

```dart
// 0.2.x
final api = await Vault.load();

// 0.3.0
final revault = await Revault.load();
```

You normally do not pass `revault` to later calls. Awaiting `Revault.load()`
installs the process-wide native runtime used by `Vault`, `Lockbox`, and
`AgentSession`.

## Open the persistent vault

Passphrases and passwords now use `SecretString`, while content and unlock keys
use `SecretBytes`. Close them to wipe their owned byte buffers:

```dart
final vaultPassphrase = SecretString.takeUtf8(await prompt.readUtf8());
try {
  final vault = Vault.open(passphrase: vaultPassphrase);
  // ...
  vault.close();
} finally {
  vaultPassphrase.close();
}
```

`SecretString.fromString` is available when an API supplies only a Dart
`String`, but that immutable source string cannot be wiped by reVault.

```dart
// 0.2.x: platform passphrase retrieval and opening were separate.
final passphrase = api.getPlatformPassword();
final vault = api.openVaultDirectory(api.defaultVaultDirectory, passphrase);

// 0.3.0: omission means use the OS credential store.
final vault = Vault.open();

// Supply the passphrase explicitly when it must not be persisted.
final vault = Vault.open(passphrase: vaultPassphrase);
```

Use the explicit creation operations when appropriate:

```dart
final vault = Vault.openOrCreate(passphrase: vaultPassphrase);
final emptyVault = Vault.replace(passphrase: vaultPassphrase);
```

`Vault.open()` never creates data. `Vault.replace()` destroys existing vault
data at the selected location.

## Create and open lockboxes

```dart
// 0.2.x
final box = api.createLockboxWithPassword(lockboxPassword);
final reopened = api.openLockboxWithPassword(archiveBytes, lockboxPassword);

// 0.3.0, host-file backed
final box = Lockbox.create(
  '/secrets/team.lbox',
  password: lockboxPassword,
);
final reopened = Lockbox.open(
  '/secrets/team.lbox',
  password: lockboxPassword,
);

// 0.3.0, in-memory
final memoryBox = Lockbox.createInMemory(password: lockboxPassword);
final memoryOpen = Lockbox.openBytes(
  archiveBytes,
  password: lockboxPassword,
);
```

To let reVault choose a remembered password or profile key from an already-open
vault:

```dart
final box = Lockbox.open('/secrets/team.lbox', vault: vault);
```

To remember a password for later lookup, store it inside an open Vault:

```dart
final lockboxId = revault.inspectLockboxFile('/secrets/team.lbox').lockboxId;
vault.rememberPassword(lockboxId, lockboxPassword);
final box = Lockbox.open('/secrets/team.lbox', vault: vault);
```

The password remains encrypted inside the Vault; it is not an independent
platform credential. With no explicit credential, `Lockbox.open(path)` opens
the default Vault with its platform-stored passphrase and then performs the
same lookup. Raw lockbox content keys are never persisted by this workflow.

`Lockbox.open` is process-local and never contacts or starts the session agent.

## Use the session agent explicitly

`LocalVault` has been removed. It was not a persistent vault; it was an
agent-backed lockbox helper. Agent state now has its own API:

```dart
final agent = AgentSession.instance;
agent.start();
agent.keepOpenWithPassword(
  '/secrets/team.lbox',
  lockboxPassword,
  duration: const Duration(minutes: 30),
);

final box = Lockbox.openFromAgent('/secrets/team.lbox');
box.close(); // releases only this process's handle

agent.closeLockbox('/secrets/team.lbox');
agent.closeAll();
agent.stop();
```

The agent caches a decrypted content key, not an open file handle. Its strongest
security use is granting time-limited access to one lockbox after an interactive
vault unlock, without retaining the vault passphrase.

## Close resources

Use `close()` for `Lockbox` and writable `Vault` objects:

```dart
try {
  // use box and vault
} finally {
  box.close();
  vault.close();
}
```

The deprecated `dispose()` aliases remain for these two types during the 0.3
series, but new code should use `close()`.

## Handle failures

Native failures now throw `RevaultException`. The exception contains structured
details and a stable category where the native operation provides them.

```dart
try {
  final box = Lockbox.open(path, password: password);
  // ...
} on RevaultException catch (error) {
  print(error.category);
  print(error.message);
  print(error.guidance);
}
```

Remove reads of `lastError` and `lastErrorDetails`; those POSIX-style properties
are no longer public.

## Rename signing identities around Profiles

The private signing identity belongs to a Vault Profile. “Owner” is retained
only for the role that identity occupies within a Lockbox:

```dart
final signingKey = vault.loadProfileSigningKey('personal');
try {
  lockbox.setOwnerSigningKey(signingKey);
} finally {
  signingKey.dispose();
}
```

Accordingly, `SigningKeyPair` and `SigningPublicKey` are now
`ProfileSigningKeyPair` and `ProfileSigningPublicKey`, and Vault and agent
methods use `ProfileSigning` in their names. `Lockbox.setOwnerSigningKey` and
`OwnerInspection` retain “Owner” because they describe the Lockbox role.

## Import one public library

Import only `package:revault_api/revault_api.dart`. The former `vault.dart`
entry point is now an implementation file under `lib/src`, so dartdoc presents
the package as one library.

## Replace string options with enums

```dart
// 0.2.x
contact.export('lockbox-pem');
box.setWorkloadProfile('bulk-import');
box.setWorkerPolicy('single', 1);
api.beginAgentActivity('open');

// 0.3.0
contact.export(KeyExportFormat.lockboxPem);
box.setWorkloadProfile(LockboxWorkload.bulkImport);
box.setWorkerPolicy(LockboxWorker.single, jobs: 1);
AgentSession.instance.beginActivity(AgentActivityKind.open);
```

## Platform credential warning

On platforms that do not enforce user presence for each credential retrieval,
remembering the vault passphrase permits unattended same-user processes to open
the vault and every lockbox for which it contains a usable credential. Closing
an agent entry does not restore an authentication boundary while that
passphrase remains available.

Future platform integrations may require user-mediated biometric or equivalent
authentication whenever the Vault credential is retrieved to open a lockbox.
Applications must feature-detect that capability when it becomes
available rather than assuming credential-store presence implies user-presence
enforcement.
