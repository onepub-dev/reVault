# reVault for JavaScript

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

The Node package selects the reVault engine for the current platform and
exposes JavaScript classes for Lockboxes, Vaults, keys, and the session agent.

## Installation and native runtime

```shell
npm install @onepub-dev/revault-api
```

The main npm package declares one optional native package for each supported
operating system and architecture. npm installs the matching package, and the
loader opens its library through Koffi. Linux packages currently require
glibc.

`Revault.load(nativeLibraryPath)` can load a native library supplied by the
application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the library
from the matching `@onepub-dev/revault-api-native-*` package. A bare name uses
the operating system library search path. Library selection is shared by the
process and must happen before the first reVault operation.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```js
import { Revault, SecretString, Vault } from '@onepub-dev/revault-api';

const runtime = await Revault.load(); // loading does not open a Vault
const signing = runtime.generateProfileSigningKeyPair();
const publicSigningKey = signing.publicKey();
const box = runtime.lockboxCreate(Buffer.alloc(32)); // Lockbox held by this process
box.setOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.addFile('/hello.txt', Buffer.from('hello\n'), false);
box.setVariable('owner', 'alice');
box.setSecretVariable('token', Buffer.from('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();
publicSigningKey.dispose(); signing.dispose();

const vaultPassphrase = new SecretString('Vault passphrase');
const persistent = Vault.openOrCreate('/tmp/revault-vault', vaultPassphrase);
persistent.close(); vaultPassphrase.close();
```

Secret callback buffers are cleared after use. `AgentSession` is explicit;
ordinary Lockbox operations never contact the agent. The hosted WebAssembly package
has the same API; the standalone browser module cannot provide OS vault or
Session Agent facilities.

## Create, open, and replace

Open methods require existing resources and never create missing state. Use
open or create only when creation is intended and replacement methods only
after an explicit destructive choice. `commit()` persists pending Lockbox
changes; `free()` or `dispose()` releases the handle and content key held by
this process without deleting the archive.

Open the resulting archive with its Lockbox password, a profile key, or a
credential resolved from the Vault. Profile signing keys identify Vault
profiles and become owner keys only after `setOwnerSigningKey`.

## Secrets, errors, and cleanup

A vault passphrase, Lockbox password, and 32-byte content key are distinct
secrets. Prefer `SecretString` and mutable `Uint8Array` values. Clear arrays
owned by the caller after use, and close objects that own secrets. JavaScript
strings cannot be wiped.

Secret callbacks receive a temporary `Uint8Array` that is cleared immediately
after return. Do not capture it, return it, or retain a view into its buffer.
Native failures throw typed errors with structured details.

Close or dispose every Vault, Lockbox, key, secret, and agent object in a
`finally` block. Loading the runtime owns no Vault state and does not require
disposal.

## Optional session agent

Ordinary Vault and Lockbox operations never start or consult the agent. Use
`AgentSession` when Lockbox keys need to be shared across processes or remain
available after the process that opened the Lockbox exits. Closing an entry
forgets a cached content key; it does
not delete a Lockbox or a credential remembered by the Vault.

## Platform credential store

The operating system credential store can hold the Vault passphrase. The
user's operating system login normally unlocks that store. After login,
another process running as that user may be able to retrieve the passphrase if
the access policy applied to the saved Vault passphrase does not require
approval for each retrieval. Exact access depends on the operating system, the
credential store configuration, and that access policy.

A process that retrieves the Vault passphrase can open the Vault. The Vault
can then provide access to Lockboxes through profile keys or remembered
Lockbox passwords. Both remain encrypted inside the Vault; they are not copied
to the operating system credential store.

Agent expiry improves memory hygiene. It is not an authentication boundary
after login if the saved Vault passphrase can be retrieved without approval.

## API documentation and support

The package's `index.d.ts` is the exact version class and method reference.
The [JavaScript conformance program](../e2e/javascript/conformance.js)
demonstrates the complete operation inventory, cleanup, and error handling. The
[example index](../API_EXAMPLES.md) tracks coverage. Report placeholder or
missing JSDoc as a binding defect.
