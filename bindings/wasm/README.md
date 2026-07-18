# reVault hosted WebAssembly API

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. This Node-hosted package runs the complete API through
a WebAssembly dispatcher while delegating native filesystem, keyring, and agent
facilities to the host package. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
npm install @onepub-dev/revault-api-wasm@0.2.0
```

```js
import { Vault, wasmDispatchCount } from '@onepub-dev/revault-api-wasm';

const vault = new Vault();
const box = vault.lockboxCreate(new Uint8Array(32));
box.setVariable('owner', 'alice');
box.setSecretVariable('token', new TextEncoder().encode('secret'));
box.withSecretVariable('token', token => token.length);
box.commit();
box.free();
console.log(wasmDispatchCount());
```

The standalone browser module supports portable lockbox/key operations only;
browsers cannot provide vault directories, an OS keyring, or a session agent.
Before creating a standalone browser lockbox, call
`set_weakened_allocation_allowed(true)` from the generated module. This is an
explicit acknowledgement that WebAssembly cannot lock or guard secret-memory
pages; the fail-closed default remains `false`.
