# reVault for TypeScript

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

Type declarations ship with the JavaScript package and cover its Lockbox,
Vault, key, and session agent classes. This directory is the strict TypeScript
conformance consumer.

## Installation and native runtime

```shell
npm install @onepub-dev/revault-api
```

TypeScript does not have a separate runtime package. Compilation produces
JavaScript that uses `@onepub-dev/revault-api`. npm installs the matching
`@onepub-dev/revault-api-native-*` optional package for the current operating
system and architecture, and the JavaScript loader opens its library through
Koffi. Linux packages currently require glibc.

`Revault.load(nativeLibraryPath?: string)` can load a native library supplied
by the application. Otherwise, reVault checks `REVAULT_LIBRARY` and then the
library supplied by the matching npm native package. A bare name uses the
operating system search path.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository.

```ts
import { Revault, SecretString, Vault } from '@onepub-dev/revault-api';

const runtime = await Revault.load();
const signing = runtime.generateProfileSigningKeyPair();
const publicSigningKey = signing.publicKey();
const box = runtime.lockboxCreate(new Uint8Array(32));
box.setOwnerSigningKey(signing); // the Profile now occupies this Lockbox's owner role
box.setVariable('owner', 'alice');
box.setSecretVariable('token', new TextEncoder().encode('secret'));
const length: number | undefined = box.withSecretVariable(
  'token',
  token => token.length,
);
box.commit();
box.free();
publicSigningKey.dispose(); signing.dispose();

const passphrase = new SecretString('Vault passphrase');
const persistent = Vault.openOrCreate('/tmp/revault-vault', passphrase);
persistent.close(); passphrase.close();
```

The callback buffer is temporary and cleared after return. Strict TypeScript
consumers should keep `strict` enabled to preserve nullable result checks.
`AgentSession` is explicit; Lockboxes opened by this process never contact it.

## Create, open, and replace

Open methods require existing resources and never create missing state. Use
open or create only when creation is acceptable and replacement APIs only
after an explicit destructive choice. `commit()` persists pending Lockbox
changes; `free()`, `dispose()`, or `close()` releases an owned handle but
does not delete the archive.

Open the resulting archive with its Lockbox password, a profile key, or a
credential resolved from the Vault. A profile signing key becomes a Lockbox
owner key only through explicit assignment.

## Secrets, errors, and ownership

A vault passphrase, Lockbox password, and 32-byte content key are separate
secrets. Use owning `SecretString`/`SecretBytes` values where available and
mutable `Uint8Array` input elsewhere. Clear buffers owned by the caller and close
owning secrets; JavaScript strings cannot be erased.

Secret callbacks receive a temporary `Uint8Array` that is cleared after the
callback returns. Its buffer must not escape. Native failures are typed and
carry structured details; keep strict null checking enabled for optional
lookups.

## Optional session agent

Normal Lockbox operations stay within this process and never start or consult
the agent. Use `AgentSession` when Lockbox keys need to be shared across
processes or remain available after the process that opened the Lockbox exits.
Closing an agent entry forgets the cached key, not the Lockbox file or a
credential stored in the Vault.

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

The installed `index.d.ts` is the matching version method contract. The
[TypeScript conformance program](../e2e/typescript/conformance.ts) demonstrates
the complete typed workflow, cleanup, and failures. The
[example index](../API_EXAMPLES.md) tracks method coverage; incomplete TSDoc is
a binding defect.
