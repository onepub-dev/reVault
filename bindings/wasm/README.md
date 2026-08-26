# reVault hosted WebAssembly API

reVault is an encrypted `Lockbox` archive and persistent `Vault` for files,
credentials, keys, and typed records. This Node-hosted package runs portable
lockbox operations through a WebAssembly dispatcher while delegating native
filesystem, keyring, and `AgentSession` facilities to the installed host
package. See the
[reVault manual](https://docs.revault.onepub.dev/).

```shell
npm install @onepub-dev/revault-api-wasm
```

The [complete method-example index](https://github.com/onepub-dev/reVault/blob/main/bindings/API_EXAMPLES.md)
is maintained in the source repository. The hosted dispatch lifecycle is covered by the TypeScript conformance runner
and the generated WebAssembly module under `generated/`.

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

`Revault` is the native-runtime entry point; it is not a persistent Vault. The
exported `Vault` is the host package's persistent encrypted store, and
`ProfileSigningKeyPair` names a profile identity. Browsers cannot provide Vault
directories, an OS keyring, or a session agent, so those operations require the
installed host package.

Before creating a standalone browser lockbox, call
`set_weakened_allocation_allowed(true)` from the generated module. This is an
explicit acknowledgement that WebAssembly cannot lock or guard secret-memory
pages; the fail-closed default remains `false`.
