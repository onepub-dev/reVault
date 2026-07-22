# reVault for TypeScript

reVault is an encrypted archive and local-vault library for files, credentials,
keys, and typed records. Type declarations ship with the JavaScript package;
this directory is its strict compile-time conformance consumer. See the
[reVault documentation](https://github.com/onepub-dev/reVault/tree/main/docs).

```shell
npm install @onepub-dev/revault-api@0.2.0
```

```ts
import { Vault } from '@onepub-dev/revault-api';

const vault = new Vault();
const box = vault.lockboxCreate(new Uint8Array(32));
box.setVariable('owner', 'alice');
box.setSecretVariable('token', new TextEncoder().encode('secret'));
const length: number | undefined = box.withSecretVariable(
  'token',
  token => token.length,
);
box.commit();
box.free();
```

The callback buffer is temporary and cleared after return. Strict TypeScript
consumers should keep `strict` enabled to preserve nullable-result checks.
