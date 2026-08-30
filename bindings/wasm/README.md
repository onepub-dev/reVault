# reVault hosted WebAssembly API

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

This package runs portable Lockbox operations through WebAssembly in Node. It
delegates filesystem, keyring, Vault, and `AgentSession` functions to the
installed host package.

## Installation and WebAssembly runtime

```shell
npm install @onepub-dev/revault-api-wasm
```

The npm package contains the generated WebAssembly module and depends on the
native `@onepub-dev/revault-api` host package. Portable archive operations run
through the WebAssembly module. Filesystem, Vault, credential store, and agent
operations run through the host package and therefore use its platform
specific npm library. A standalone browser build can use only the WebAssembly
operations and credentials supplied directly by the application.

The [complete method example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository. The TypeScript conformance runner and
the generated WebAssembly module under `generated/` cover the hosted dispatch
lifecycle.

```js
import { Revault, wasmDispatchCount } from '@onepub-dev/revault-api-wasm';

const runtime = new Revault();
const signing = runtime.generateProfileSigningKeyPair();
const publicSigningKey = signing.publicKey();
const box = runtime.lockboxCreate(new Uint8Array(32));
box.setOwnerSigningKey(signing); // Profile becomes this Lockbox's owner
box.setVariable('owner', 'alice');
box.setSecretVariable('token', new TextEncoder().encode('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();
publicSigningKey.dispose(); signing.dispose();
console.log(wasmDispatchCount());
```

`Revault` is the native runtime entry point; it is not a persistent Vault. The
exported `Vault` is the host package's persistent encrypted store, and
`ProfileSigningKeyPair` names a profile identity. Browsers cannot provide Vault
directories, a platform credential store, or a Session Agent, so those operations require the
installed host package.

Before creating a standalone browser lockbox, call
`set_weakened_allocation_allowed(true)` from the generated module. This is an
explicit acknowledgement that WebAssembly cannot lock or guard pages that hold
secrets. By default, the package refuses to run when this protection is absent.

## Core API concepts

`Revault` enters the hosted WASM runtime and `Lockbox` owns portable archive
state. The host provides Vault and Agent facilities separately. Dispose each
exported owner independently.

## API documentation and support

The npm package ships public declarations for editors and documentation tools.
The [method examples](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md),
TypeScript conformance runner, and hosted WASM tests cover the public
inventory. Browser users must also read the reduced memory protection warning
above.

## Hosted and browser capabilities

The hosted Node package can delegate filesystem, Vault, operating system
credential store, and session agent operations to its installed host package.
A standalone browser module cannot provide those services. Browser code can
operate only on explicitly supplied Lockbox bytes and credentials. The
application is responsible for storing the resulting archive.

Open operations never create missing state. Creation and replacement remain
separate explicit choices, and `commit()` must occur before exporting updated
Lockbox bytes. Disposing a handle releases the content key that it held in this
process; it does not persist or delete application storage.

## Secrets, errors, and cleanup

A vault passphrase, Lockbox password, and 32-byte content key are different
secrets. Use mutable `Uint8Array` values, clear buffers owned by the caller
after use, and do not convert secrets to JavaScript strings. Buffers passed to
secret callbacks are cleared after return and must not escape.

WebAssembly linear memory cannot provide the locked and guarded pages used by
the native runtime. The explicit reduced allocation protection choice acknowledges that
difference; it does not make browser memory equivalent to native protected
memory.

## Agent and platform credentials

The browser module has no session agent or operating system credential store.
In Node, use the hosted agent when Lockbox keys need to be shared across
processes or remain available after the process that opened the Lockbox exits.
Ordinary Lockbox operations never contact it. Closing a hosted agent entry
forgets the cached key, not a Lockbox or credentials stored in the Vault.

The hosted Node package delegates credential storage to the host package. The
user's operating system login normally unlocks that store. After login,
another process running as that user may be able to retrieve the saved Vault
passphrase without approval. Exact access depends on the operating system, the
credential store configuration, and the access policy applied to the saved
Vault passphrase.

A process that retrieves the Vault passphrase can open the Vault. The Vault
can then provide access to Lockboxes through profile keys or remembered
Lockbox passwords. Both remain encrypted inside the Vault; they are not copied
to the operating system credential store. Agent expiry is not an
authentication boundary if the saved Vault passphrase can be retrieved again
without approval.

The shipped declarations are the exact version method contract. Placeholder
TSDoc/JSDoc is a binding defect.
